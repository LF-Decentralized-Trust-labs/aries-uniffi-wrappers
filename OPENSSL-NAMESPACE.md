# Anoncreds OpenSSL namespace proof of concept

This proof of concept builds the anoncreds `0.3.1` iOS static archive with a private
OpenSSL namespace. It is intended to prove that a Kotlin/Native consumer can
embed anoncreds in its KLIB without requiring a separate dynamic XCFramework or
depending on application link order.

## Proof-of-concept contract

- Source: immutable tag `0.3.1`, commit
  `c4d4eea1abf17ac66a90ba2216801c23ecaad53c`
- Rust toolchain: `1.85.1`
- Apple targets: `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Minimum iOS version: `15.0`
- OpenSSL namespace: `anoncreds_ossl_*`
- UniFFI exports: unchanged from upstream

The build first compiles anoncreds and its vendored OpenSSL from source. It
derives a rename map from every external definition in the resulting
`libcrypto.a` and `libssl.a`, then applies that map to the complete
`libanoncreds_uniffi.a`. Applying the map to the final archive updates both
definitions and internal references.

## Build

Install the pinned toolchain and Apple targets:

```bash
rustup toolchain install 1.85.1 --profile minimal
rustup target add --toolchain 1.85.1 \
  aarch64-apple-ios aarch64-apple-ios-sim
rustup component add --toolchain 1.85.1 llvm-tools-preview
```

Run the deterministic builder from the repository root:

```bash
python3 scripts/build_anoncreds_openssl_namespaced.py \
  --output-dir build/openssl-namespaced/output
```

The output manifest records the source commit, tool versions, input and output
hashes, symbol-map hashes, symbol counts, and UniFFI export counts for both
Apple targets.

## Required verification

The builder fails unless all of these conditions hold:

1. The anoncreds source tree exactly matches upstream `0.3.1`.
2. Every OpenSSL global definition and reference represented by the vendored
   libraries is absent under its unprefixed name in the final archive.
3. Every mapped `anoncreds_ossl_*` symbol exists in the final archive.
4. The anoncreds UniFFI export set is exactly unchanged by the transform.
5. Repeating the pinned LLVM transformation produces the same archive hash.

Any release integration must additionally compare the UniFFI surface to the
published upstream KLIB and verify that non-Apple artifacts remain unchanged.
