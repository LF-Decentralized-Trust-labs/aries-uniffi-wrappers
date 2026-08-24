#!/usr/bin/env python3
"""Namespace vendored OpenSSL symbols in an anoncreds Apple static archive.

The archive produced by Cargo contains the object members from vendored
libcrypto and libssl. This tool derives the rename map from those exact
libraries, verifies that every definition in the final archive has the same
member provenance and multiplicity, and applies one map to the complete
archive so definitions and internal references move together.
"""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Iterable


OPENSSL_PREFIX = "_anoncreds_ossl_"
MACHO_SYMBOL = re.compile(r"^_[A-Za-z0-9_$.]+$")
NM_ARCHIVE_RECORD = re.compile(
    r"^(?P<archive>.+)\[(?P<member>[^]]+)]:\s+"
    r"(?P<symbol>\S+)\s+(?P<kind>\S+)"
)
UNIFFI_SYMBOL = re.compile(
    r"^_(?:"
    r"ffi_anoncreds_uniffi_|"
    r"uniffi_anoncreds_uniffi_|"
    r"UNIFFI_META_ANONCREDS_UNIFFI_|"
    r"UNIFFI_META_NAMESPACE_ANONCREDS_UNIFFI$|"
    r"UNIFFI_META_UDL_ANONCREDS_UNIFFI$"
    r")"
)

SymbolProvenance = dict[str, collections.Counter[str]]


def run(command: list[str], *, capture: bool = False) -> str:
    result = subprocess.run(
        command,
        check=True,
        text=True,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )
    return result.stdout.strip() if capture else ""


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def resolve_llvm_tool(name: str, environment_name: str) -> Path:
    configured = os.environ.get(environment_name)
    if configured:
        path = Path(configured).expanduser().resolve()
        if path.is_file():
            return path
        raise SystemExit(f"{environment_name} does not point to a file: {path}")

    rustc = shutil.which("rustc")
    if not rustc:
        rustup_candidates = [
            os.environ.get("RUSTUP_BIN"),
            shutil.which("rustup"),
            "/opt/homebrew/opt/rustup/bin/rustup",
        ]
        rustup = next(
            (
                candidate
                for candidate in rustup_candidates
                if candidate and Path(candidate).is_file()
            ),
            None,
        )
        if rustup:
            try:
                rustc = run([rustup, "which", "rustc"], capture=True)
            except subprocess.CalledProcessError:
                rustc = None
    if rustc:
        verbose = run([rustc, "-vV"], capture=True)
        host = next(
            (
                line.split(":", 1)[1].strip()
                for line in verbose.splitlines()
                if line.startswith("host:")
            ),
            None,
        )
        sysroot = Path(run([rustc, "--print", "sysroot"], capture=True))
        if host:
            candidate = sysroot / "lib" / "rustlib" / host / "bin" / name
            if candidate.is_file():
                return candidate

    fallback = shutil.which(name)
    if fallback:
        return Path(fallback).resolve()

    raise SystemExit(
        f"{name} is required. Install the llvm-tools-preview component for "
        "the active Rust toolchain or set "
        f"{environment_name}."
    )


def nm_records(
    llvm_nm: Path,
    paths: Iterable[Path],
    *,
    defined_only: bool = False,
    undefined_only: bool = False,
) -> SymbolProvenance:
    command = [str(llvm_nm), "-A", "-g", "--format=posix"]
    if defined_only:
        command.append("--defined-only")
    if undefined_only:
        command.append("--undefined-only")
    command.extend(str(path) for path in paths)

    records: SymbolProvenance = collections.defaultdict(collections.Counter)
    for line in run(command, capture=True).splitlines():
        match = NM_ARCHIVE_RECORD.match(line)
        if not match:
            continue
        symbol = match.group("symbol")
        if MACHO_SYMBOL.fullmatch(symbol):
            records[symbol][match.group("member")] += 1
    return dict(records)


def find_openssl_libraries(release_dir: Path) -> list[tuple[Path, Path]]:
    candidates = sorted(
        release_dir.glob(
            "build/openssl-sys-*/out/openssl-build/install/lib/libcrypto.a"
        )
    )
    if not candidates:
        raise SystemExit(
            f"No vendored libcrypto.a was found under {release_dir}"
        )
    pairs: list[tuple[Path, Path]] = []
    for libcrypto in candidates:
        libssl = libcrypto.with_name("libssl.a")
        if not libssl.is_file():
            raise SystemExit(f"Vendored libssl.a is missing: {libssl}")
        pairs.append((libcrypto, libssl))
    return pairs


def symbol_mapping(symbols: Iterable[str]) -> dict[str, str]:
    return {
        symbol: f"{OPENSSL_PREFIX}{symbol.removeprefix('_')}"
        for symbol in sorted(symbols)
    }


def compare_provenance(
    expected: SymbolProvenance,
    actual: SymbolProvenance,
    mapping: dict[str, str],
    *,
    namespaced: bool,
) -> list[str]:
    mismatches: list[str] = []
    for source, expected_members in expected.items():
        symbol = mapping[source] if namespaced else source
        if actual.get(symbol, collections.Counter()) != expected_members:
            mismatches.append(symbol)
    return mismatches


def environment_receipt(llvm_nm: Path, llvm_objcopy: Path) -> dict[str, object]:
    xcrun = shutil.which("xcrun")
    xcodebuild = shutil.which("xcodebuild")
    sw_vers = shutil.which("sw_vers")
    if not xcrun or not xcodebuild or not sw_vers:
        raise SystemExit("The Apple build environment is required for namespacing")
    clang = run([xcrun, "--find", "clang"], capture=True)
    return {
        "macOsVersion": run([sw_vers, "-productVersion"], capture=True),
        "hostArchitecture": platform.machine(),
        "xcode": run([xcodebuild, "-version"], capture=True).splitlines(),
        "clang": run([clang, "--version"], capture=True).splitlines(),
        "sdkVersions": {
            sdk: run([xcrun, "--sdk", sdk, "--show-sdk-version"], capture=True)
            for sdk in ("iphoneos", "iphonesimulator", "macosx")
        },
        "llvmNm": run([str(llvm_nm), "--version"], capture=True).splitlines(),
        "llvmObjcopy": run(
            [str(llvm_objcopy), "--version"], capture=True
        ).splitlines(),
    }


def validate_namespaced_archive(
    llvm_nm: Path,
    archive: Path,
    vendor_definitions: SymbolProvenance,
    mapping: dict[str, str],
) -> tuple[SymbolProvenance, SymbolProvenance, set[str]]:
    definitions = nm_records(llvm_nm, (archive,), defined_only=True)
    references = nm_records(llvm_nm, (archive,), undefined_only=True)
    unprefixed_definitions = set(vendor_definitions) & set(definitions)
    unprefixed_references = set(vendor_definitions) & set(references)
    expected_prefixed = set(mapping.values())
    actual_prefixed = {
        symbol for symbol in definitions if symbol.startswith(OPENSSL_PREFIX)
    }
    missing_prefixed = expected_prefixed - actual_prefixed
    unexpected_prefixed = actual_prefixed - expected_prefixed
    provenance_mismatches = compare_provenance(
        vendor_definitions, definitions, mapping, namespaced=True
    )

    if unprefixed_definitions or unprefixed_references:
        raise SystemExit(
            "Unprefixed vendored OpenSSL symbols remain: "
            f"definitions={len(unprefixed_definitions)}, "
            f"references={len(unprefixed_references)}"
        )
    if missing_prefixed or unexpected_prefixed or provenance_mismatches:
        raise SystemExit(
            "Namespaced symbol surface or provenance mismatch: "
            f"missing={len(missing_prefixed)}, "
            f"unexpected={len(unexpected_prefixed)}, "
            f"provenance={len(provenance_mismatches)}"
        )
    return definitions, references, actual_prefixed


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--libcrypto", type=Path)
    parser.add_argument("--libssl", type=Path)
    parser.add_argument("--minimum-symbol-count", type=int, default=1000)
    parser.add_argument("--skip-uniffi-check", action="store_true")
    args = parser.parse_args()

    archive = args.archive.resolve()
    receipt = args.receipt.resolve()
    if not archive.is_file():
        raise SystemExit(f"Anoncreds archive is missing: {archive}")
    if bool(args.libcrypto) != bool(args.libssl):
        raise SystemExit("--libcrypto and --libssl must be supplied together")
    if args.skip_uniffi_check and os.environ.get("ANONCREDS_NAMESPACE_TESTING") != "1":
        raise SystemExit("--skip-uniffi-check is restricted to the regression fixture")

    if args.libcrypto:
        library_candidates = [(args.libcrypto.resolve(), args.libssl.resolve())]
    else:
        library_candidates = find_openssl_libraries(archive.parent)
    for pair in library_candidates:
        for library in pair:
            if not library.is_file():
                raise SystemExit(f"Vendored OpenSSL archive is missing: {library}")

    llvm_nm = resolve_llvm_tool("llvm-nm", "LLVM_NM")
    llvm_objcopy = resolve_llvm_tool("llvm-objcopy", "LLVM_OBJCOPY")
    baseline_definitions = nm_records(llvm_nm, (archive,), defined_only=True)
    baseline_references = nm_records(llvm_nm, (archive,), undefined_only=True)
    matching_candidates: list[SymbolProvenance] = []
    for libcrypto, libssl in library_candidates:
        candidate_definitions = nm_records(
            llvm_nm, (libcrypto, libssl), defined_only=True
        )
        if len(candidate_definitions) < args.minimum_symbol_count:
            continue
        candidate_mapping = symbol_mapping(candidate_definitions)
        matches_source = not compare_provenance(
            candidate_definitions,
            baseline_definitions,
            candidate_mapping,
            namespaced=False,
        )
        matches_namespaced = not compare_provenance(
            candidate_definitions,
            baseline_definitions,
            candidate_mapping,
            namespaced=True,
        )
        if matches_source or matches_namespaced:
            matching_candidates.append(candidate_definitions)

    if not matching_candidates:
        raise SystemExit(
            "No vendored OpenSSL output has the same symbol provenance and "
            "multiplicity as the final anoncreds archive"
        )
    vendor_definitions = matching_candidates[0]
    if any(candidate != vendor_definitions for candidate in matching_candidates[1:]):
        raise SystemExit(
            "Multiple non-equivalent vendored OpenSSL outputs match the final archive"
        )
    mapping = symbol_mapping(vendor_definitions)
    expected_prefixed = set(mapping.values())
    initial_uniffi_exports = {
        symbol for symbol in baseline_definitions if UNIFFI_SYMBOL.match(symbol)
    }
    if not args.skip_uniffi_check and len(initial_uniffi_exports) < 300:
        raise SystemExit(
            f"Unexpected UniFFI export surface: {len(initial_uniffi_exports)}"
        )
    already_namespaced = expected_prefixed <= set(baseline_definitions)
    source_archive_sha256: str | None = None
    replay_verified = False

    if already_namespaced:
        state = "already_namespaced"
    else:
        prefixed_before = {
            symbol
            for symbol in baseline_definitions | baseline_references
            if symbol.startswith(OPENSSL_PREFIX)
        }
        provenance_mismatches = compare_provenance(
            vendor_definitions,
            baseline_definitions,
            mapping,
            namespaced=False,
        )
        missing_definitions = set(vendor_definitions) - set(baseline_definitions)
        if prefixed_before or missing_definitions or provenance_mismatches:
            raise SystemExit(
                "Refusing to transform an archive with mixed or unexpected provenance: "
                f"prefixed={len(prefixed_before)}, "
                f"missing={len(missing_definitions)}, "
                f"provenance={len(provenance_mismatches)}"
            )

        source_archive_sha256 = sha256(archive)
        with tempfile.TemporaryDirectory(
            prefix="anoncreds-openssl-namespace-", dir=archive.parent
        ) as temporary_directory:
            temporary = Path(temporary_directory)
            symbol_map = temporary / "openssl-symbol-map.txt"
            symbol_map.write_text(
                "".join(
                    f"{source} {destination}\n"
                    for source, destination in mapping.items()
                ),
                encoding="utf-8",
            )
            transformed = temporary / archive.name
            replay = temporary / f"{archive.stem}.replay{archive.suffix}"
            command = [
                str(llvm_objcopy),
                f"--redefine-syms={symbol_map}",
                str(archive),
            ]
            run([*command, str(transformed)])
            run([*command, str(replay)])
            if sha256(transformed) != sha256(replay):
                raise SystemExit(
                    "The LLVM symbol transformation replay produced a different archive"
                )
            transformed_definitions, _, _ = validate_namespaced_archive(
                llvm_nm, transformed, vendor_definitions, mapping
            )
            transformed_uniffi_exports = {
                symbol
                for symbol in transformed_definitions
                if UNIFFI_SYMBOL.match(symbol)
            }
            if transformed_uniffi_exports != initial_uniffi_exports:
                raise SystemExit(
                    "The anoncreds UniFFI export surface changed during namespacing"
                )
            replay_verified = True
            os.replace(transformed, archive)
        state = "transformed"

    definitions, references, prefixed = validate_namespaced_archive(
        llvm_nm, archive, vendor_definitions, mapping
    )
    uniffi_exports = {
        symbol for symbol in definitions if UNIFFI_SYMBOL.match(symbol)
    }
    if not args.skip_uniffi_check and len(uniffi_exports) < 300:
        raise SystemExit(f"Unexpected UniFFI export surface: {len(uniffi_exports)}")
    if uniffi_exports != initial_uniffi_exports:
        raise SystemExit(
            "The anoncreds UniFFI export surface changed during namespacing: "
            f"before={len(initial_uniffi_exports)}, after={len(uniffi_exports)}"
        )

    mapping_text = "".join(
        f"{source} {destination}\n" for source, destination in mapping.items()
    ).encode()
    receipt.parent.mkdir(parents=True, exist_ok=True)
    payload: dict[str, object] = {
        "schema": 1,
        "target": args.target,
        "state": state,
        "archiveSha256": sha256(archive),
        "sourceArchiveSha256": source_archive_sha256,
        "symbolMapSha256": hashlib.sha256(mapping_text).hexdigest(),
        "vendoredSymbolCount": len(vendor_definitions),
        "matchingVendoredOutputCount": len(matching_candidates),
        "namespacedDefinitionCount": len(prefixed),
        "definitionProvenanceMismatchCount": 0,
        "unprefixedDefinitionCount": 0,
        "unprefixedReferenceCount": 0,
        "uniffiExportCount": len(uniffi_exports),
        "uniffiExportSurfaceUnchanged": True,
        "symbolTransformationReplayVerified": replay_verified,
        "environment": environment_receipt(llvm_nm, llvm_objcopy),
    }
    receipt.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(payload, indent=2))


if __name__ == "__main__":
    main()
