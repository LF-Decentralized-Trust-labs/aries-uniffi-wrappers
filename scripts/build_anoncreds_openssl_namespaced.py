#!/usr/bin/env python3
"""Build iOS anoncreds archives with a private OpenSSL symbol namespace.

The Rust and vendored OpenSSL sources are rebuilt from the immutable 0.3.1
source line.  The pinned Rust LLVM toolchain then applies one complete symbol
map to the final static archive, rewriting both definitions and references.
Only OpenSSL symbols are changed; the anoncreds UniFFI ABI remains intact.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
from pathlib import Path
from typing import Iterable


OPENSSL_PREFIX = "_anoncreds_ossl_"
RUST_TOOLCHAIN = "1.85.1"
UPSTREAM_COMMIT = "c4d4eea1abf17ac66a90ba2216801c23ecaad53c"
MINIMUM_IOS_VERSION = "15.0"

TARGETS = {
    "ios_arm64": {
        "rust": "aarch64-apple-ios",
        "minimumFlag": f"-miphoneos-version-min={MINIMUM_IOS_VERSION}",
    },
    "ios_simulator_arm64": {
        "rust": "aarch64-apple-ios-sim",
        "minimumFlag": f"-mios-simulator-version-min={MINIMUM_IOS_VERSION}",
    },
}

MACHO_SYMBOL = re.compile(r"^_[A-Za-z0-9_$.]+$")
UNIFFI_SYMBOL = re.compile(
    r"^_(?:"
    r"ffi_anoncreds_uniffi_|"
    r"uniffi_anoncreds_uniffi_|"
    r"UNIFFI_META_ANONCREDS_UNIFFI_|"
    r"UNIFFI_META_NAMESPACE_ANONCREDS_UNIFFI$|"
    r"UNIFFI_META_UDL_ANONCREDS_UNIFFI$"
    r")"
)


def run(
    command: list[str],
    *,
    cwd: Path,
    environment: dict[str, str] | None = None,
    capture: bool = False,
) -> str:
    result = subprocess.run(
        command,
        check=True,
        cwd=cwd,
        env=environment,
        text=True,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )
    return result.stdout if capture else ""


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def find_rustup() -> Path:
    configured = os.environ.get("RUSTUP_BIN")
    candidates = [
        Path(configured) if configured else None,
        Path(shutil.which("rustup")) if shutil.which("rustup") else None,
        Path("/opt/homebrew/opt/rustup/bin/rustup"),
    ]
    for candidate in candidates:
        if candidate and candidate.is_file():
            return candidate.resolve()
    raise SystemExit("rustup is required to build the namespaced anoncreds archive")


def rust_tool(rustup: Path, source_root: Path, name: str) -> Path:
    resolved = run(
        [str(rustup), "which", "--toolchain", RUST_TOOLCHAIN, name],
        cwd=source_root,
        capture=True,
    ).strip()
    path = Path(resolved)
    if not path.is_file():
        raise SystemExit(f"Rust tool is missing: {path}")
    return path


def llvm_tools(rustc: Path, source_root: Path) -> tuple[Path, Path, str]:
    verbose = run([str(rustc), "-vV"], cwd=source_root, capture=True)
    host = next(
        (line.split(":", 1)[1].strip() for line in verbose.splitlines() if line.startswith("host:")),
        None,
    )
    if not host:
        raise SystemExit("Unable to resolve the Rust host target")
    sysroot = Path(
        run([str(rustc), "--print", "sysroot"], cwd=source_root, capture=True).strip()
    )
    tools = sysroot / "lib" / "rustlib" / host / "bin"
    llvm_nm = tools / "llvm-nm"
    llvm_objcopy = tools / "llvm-objcopy"
    for tool in (llvm_nm, llvm_objcopy):
        if not tool.is_file():
            raise SystemExit(
                f"Pinned LLVM tool is missing: {tool}. "
                f"Install llvm-tools-preview for Rust {RUST_TOOLCHAIN}."
            )
    llvm_version = run(
        [str(llvm_objcopy), "--version"], cwd=source_root, capture=True
    ).strip()
    return llvm_nm, llvm_objcopy, llvm_version


def external_symbols(
    llvm_nm: Path,
    paths: Iterable[Path],
    *,
    defined_only: bool = False,
    undefined_only: bool = False,
    source_root: Path,
) -> set[str]:
    command = [str(llvm_nm), "--extern-only", "--format=posix"]
    if defined_only:
        command.append("--defined-only")
    if undefined_only:
        command.append("--undefined-only")
    command.extend(str(path) for path in paths)
    output = run(command, cwd=source_root, capture=True)
    symbols: set[str] = set()
    for line in output.splitlines():
        fields = line.split()
        if len(fields) >= 2 and MACHO_SYMBOL.fullmatch(fields[0]) and fields[1].isalpha():
            symbols.add(fields[0])
    return symbols


def ensure_source_matches_upstream(source_root: Path) -> int:
    resolved = run(
        ["git", "rev-parse", "0.3.1^{commit}"], cwd=source_root, capture=True
    ).strip()
    if resolved != UPSTREAM_COMMIT:
        raise SystemExit(
            f"Unexpected 0.3.1 source commit: {resolved}; expected {UPSTREAM_COMMIT}"
        )
    source_changes = subprocess.run(
        ["git", "diff", "--quiet", "0.3.1", "--", "anoncreds"], cwd=source_root
    )
    staged_changes = subprocess.run(
        ["git", "diff", "--cached", "--quiet", "0.3.1", "--", "anoncreds"],
        cwd=source_root,
    )
    if source_changes.returncode or staged_changes.returncode:
        raise SystemExit("anoncreds sources differ from the immutable upstream 0.3.1 tag")
    return int(
        run(
            ["git", "show", "-s", "--format=%ct", UPSTREAM_COMMIT],
            cwd=source_root,
            capture=True,
        ).strip()
    )


def find_openssl_libraries(release_dir: Path) -> tuple[Path, Path]:
    candidates = sorted(
        release_dir.glob(
            "build/openssl-sys-*/out/openssl-build/install/lib/libcrypto.a"
        )
    )
    if len(candidates) != 1:
        raise SystemExit(
            f"Expected one vendored libcrypto.a under {release_dir}, found {len(candidates)}"
        )
    libcrypto = candidates[0]
    libssl = libcrypto.with_name("libssl.a")
    if not libssl.is_file():
        raise SystemExit(f"Vendored libssl.a is missing: {libssl}")
    return libcrypto, libssl


def write_symbol_map(path: Path, symbols: set[str]) -> dict[str, str]:
    if len(symbols) < 1000:
        raise SystemExit(f"Unexpectedly small OpenSSL symbol surface: {len(symbols)}")
    mapping = {
        symbol: f"{OPENSSL_PREFIX}{symbol.removeprefix('_')}" for symbol in sorted(symbols)
    }
    path.write_text(
        "".join(f"{source} {destination}\n" for source, destination in mapping.items()),
        encoding="utf-8",
    )
    return mapping


def build_target(
    *,
    key: str,
    target: dict[str, str],
    source_root: Path,
    cargo: Path,
    rustc: Path,
    llvm_nm: Path,
    llvm_objcopy: Path,
    cargo_home: Path,
    build_root: Path,
    output_root: Path,
    source_date_epoch: int,
) -> tuple[dict[str, object], set[str]]:
    rust_target = target["rust"]
    target_dir = build_root / rust_target
    release_dir = target_dir / rust_target / "release"
    environment = os.environ.copy()
    environment.update(
        {
            "CARGO_HOME": str(cargo_home),
            "CARGO_TARGET_DIR": str(target_dir),
            "RUSTC": str(rustc),
            "IPHONEOS_DEPLOYMENT_TARGET": MINIMUM_IOS_VERSION,
            "SOURCE_DATE_EPOCH": str(source_date_epoch),
            "ZERO_AR_DATE": "1",
            "RUSTFLAGS": " ".join(
                [
                    "-C",
                    f"link-arg={target['minimumFlag']}",
                    "--remap-path-prefix",
                    f"{source_root}=/src/aries-uniffi-wrappers",
                    "--remap-path-prefix",
                    f"{cargo_home}=/cargo",
                ]
            ),
        }
    )
    run(
        [
            str(cargo),
            "build",
            "--release",
            "--locked",
            "--target",
            rust_target,
            "--manifest-path",
            str(source_root / "anoncreds/Cargo.toml"),
        ],
        cwd=source_root,
        environment=environment,
    )

    baseline = release_dir / "libanoncreds_uniffi.a"
    if not baseline.is_file():
        raise SystemExit(f"Source build did not produce {baseline}")
    libcrypto, libssl = find_openssl_libraries(release_dir)
    openssl_symbols = external_symbols(
        llvm_nm,
        (libcrypto, libssl),
        defined_only=True,
        source_root=source_root,
    )

    destination_dir = output_root / key
    destination_dir.mkdir(parents=True)
    symbol_map = destination_dir / "openssl-symbol-map.txt"
    mapping = write_symbol_map(symbol_map, openssl_symbols)
    namespaced = destination_dir / "libanoncreds_uniffi.a"
    run(
        [
            str(llvm_objcopy),
            f"--redefine-syms={symbol_map}",
            str(baseline),
            str(namespaced),
        ],
        cwd=source_root,
    )

    # Re-run the transformation to prove that the pinned LLVM step is stable.
    replay = destination_dir / "libanoncreds_uniffi.replay.a"
    run(
        [
            str(llvm_objcopy),
            f"--redefine-syms={symbol_map}",
            str(baseline),
            str(replay),
        ],
        cwd=source_root,
    )
    if sha256(namespaced) != sha256(replay):
        raise SystemExit(f"Namespaced archive transformation is not reproducible for {key}")
    replay.unlink()

    defined = external_symbols(
        llvm_nm, (namespaced,), defined_only=True, source_root=source_root
    )
    undefined = external_symbols(
        llvm_nm, (namespaced,), undefined_only=True, source_root=source_root
    )
    all_symbols = defined | undefined
    unprefixed_definitions = openssl_symbols & defined
    unprefixed_references = openssl_symbols & undefined
    expected_prefixed = set(mapping.values())
    missing_prefixed = expected_prefixed - all_symbols
    unexpected_prefixed = {
        symbol for symbol in all_symbols if symbol.startswith(OPENSSL_PREFIX)
    } - expected_prefixed
    if unprefixed_definitions or unprefixed_references:
        raise SystemExit(
            f"Unprefixed OpenSSL symbols remain in {key}: "
            f"definitions={len(unprefixed_definitions)}, "
            f"references={len(unprefixed_references)}"
        )
    if missing_prefixed or unexpected_prefixed:
        raise SystemExit(
            f"Namespaced OpenSSL surface mismatch in {key}: "
            f"missing={len(missing_prefixed)}, unexpected={len(unexpected_prefixed)}"
        )

    baseline_uniffi = {
        symbol
        for symbol in external_symbols(
            llvm_nm, (baseline,), defined_only=True, source_root=source_root
        )
        if UNIFFI_SYMBOL.match(symbol)
    }
    namespaced_uniffi = {symbol for symbol in defined if UNIFFI_SYMBOL.match(symbol)}
    if len(baseline_uniffi) < 300 or baseline_uniffi != namespaced_uniffi:
        raise SystemExit(
            f"UniFFI export surface changed in {key}: "
            f"before={len(baseline_uniffi)}, after={len(namespaced_uniffi)}"
        )

    target_manifest: dict[str, object] = {
        "rustTarget": rust_target,
        "archive": str(namespaced.relative_to(output_root)),
        "archiveSha256": sha256(namespaced),
        "sourceArchiveSha256": sha256(baseline),
        "symbolMap": str(symbol_map.relative_to(output_root)),
        "symbolMapSha256": sha256(symbol_map),
        "opensslSymbolCount": len(openssl_symbols),
        "namespacedSymbolCount": len(expected_prefixed),
        "unprefixedDefinitionCount": 0,
        "unprefixedReferenceCount": 0,
        "uniffiExportCount": len(namespaced_uniffi),
    }
    return target_manifest, openssl_symbols


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--source-root",
        type=Path,
        help="Immutable aries-uniffi-wrappers 0.3.1 checkout",
    )
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument(
        "--cargo-home",
        type=Path,
        help="Cargo cache directory; defaults to build/openssl-namespaced/cargo-home",
    )
    parser.add_argument(
        "--build-root",
        type=Path,
        help="Cargo target directory root; defaults to build/openssl-namespaced/cargo-target",
    )
    args = parser.parse_args()

    source_root = (
        args.source_root.resolve()
        if args.source_root
        else Path(__file__).resolve().parents[1]
    )
    if not (source_root / "anoncreds/Cargo.toml").is_file():
        raise SystemExit(f"Invalid aries-uniffi-wrappers source root: {source_root}")
    output_root = args.output_dir.resolve()
    if output_root.exists() and any(output_root.iterdir()):
        raise SystemExit(f"Output directory must be empty: {output_root}")
    output_root.mkdir(parents=True, exist_ok=True)
    cargo_home = (
        args.cargo_home.resolve()
        if args.cargo_home
        else source_root / "build/openssl-namespaced/cargo-home"
    )
    build_root = (
        args.build_root.resolve()
        if args.build_root
        else source_root / "build/openssl-namespaced/cargo-target"
    )
    cargo_home.mkdir(parents=True, exist_ok=True)
    build_root.mkdir(parents=True, exist_ok=True)

    source_date_epoch = ensure_source_matches_upstream(source_root)
    rustup = find_rustup()
    cargo = rust_tool(rustup, source_root, "cargo")
    rustc = rust_tool(rustup, source_root, "rustc")
    llvm_nm, llvm_objcopy, llvm_version = llvm_tools(rustc, source_root)
    rust_version = run([str(rustc), "--version"], cwd=source_root, capture=True).strip()

    target_manifests: dict[str, dict[str, object]] = {}
    symbol_sets: list[set[str]] = []
    for key, target in TARGETS.items():
        target_manifest, openssl_symbols = build_target(
            key=key,
            target=target,
            source_root=source_root,
            cargo=cargo,
            rustc=rustc,
            llvm_nm=llvm_nm,
            llvm_objcopy=llvm_objcopy,
            cargo_home=cargo_home,
            build_root=build_root,
            output_root=output_root,
            source_date_epoch=source_date_epoch,
        )
        target_manifests[key] = target_manifest
        symbol_sets.append(openssl_symbols)

    shared_symbols = symbol_sets[0] & symbol_sets[1]
    target_specific_symbols = {
        key: len(symbol_sets[index] - shared_symbols)
        for index, key in enumerate(TARGETS)
    }

    manifest = {
        "schema": 1,
        "upstreamVersion": "0.3.1",
        "upstreamCommit": UPSTREAM_COMMIT,
        "opensslPrefix": OPENSSL_PREFIX.removeprefix("_"),
        "minimumIosVersion": MINIMUM_IOS_VERSION,
        "sourceDateEpoch": source_date_epoch,
        "rustToolchain": RUST_TOOLCHAIN,
        "rustVersion": rust_version,
        "llvmVersion": llvm_version.splitlines(),
        "cargoLockSha256": sha256(source_root / "anoncreds/Cargo.lock"),
        "sharedOpenSslSymbolCount": len(shared_symbols),
        "targetSpecificOpenSslSymbolCount": target_specific_symbols,
        "targets": target_manifests,
    }
    manifest_path = output_root / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    main()
