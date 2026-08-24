# Anoncreds OpenSSL symbol namespace

The anoncreds Apple archives statically contain the vendored OpenSSL objects.
Those objects use the private `anoncreds_ossl_*` symbol namespace so an
application can safely link another static OpenSSL provider without depending
on library order.

This is part of the normal Apple build path. It does not introduce a separate
artifact version, public Kotlin API, UniFFI ABI, or runtime framework.

## Published target coverage

The namespace pass runs for every Apple target published by the Kotlin module:

- `aarch64-apple-ios` (`iosArm64`)
- `aarch64-apple-ios-sim` (`iosSimulatorArm64`)
- `x86_64-apple-ios` (`iosX64`)
- `aarch64-apple-darwin` (`macosArm64`)
- `x86_64-apple-darwin` (`macosX64`)

The Swift XCFramework builder applies the same pass to these five archives.
Each build path retains its existing deployment target: the KMP Apple mobile
configuration remains iOS 10.0, while the Swift package retains its existing
iOS 15 requirement.

## Transformation contract

`scripts/namespace_anoncreds_openssl.py` performs the following fail-closed
steps after Cargo builds `libanoncreds_uniffi.a`:

1. Locate the exact vendored `libcrypto.a` and `libssl.a` produced by
   `openssl-sys` for that target.
2. Read global definitions with `llvm-nm -A`, retaining archive-member
   provenance and definition multiplicity.
3. Require each source definition to have exactly the same provenance in the
   final anoncreds archive. This prevents a same-named definition from an
   unrelated object from being renamed accidentally.
4. Apply the complete map to the final archive with LLVM `--redefine-syms`,
   updating definitions and internal references together.
5. Reject remaining unprefixed definitions or references, missing or
   unexpected prefixed definitions, provenance changes, or a changed UniFFI
   surface.

The pass is idempotent. An already namespaced archive is verified without a
second rename; a Cargo task that rewrites its output is transformed again.

## Toolchain and build environment

The repository does not pin a root Rust toolchain for this feature. The
release workflow selects its Rust version explicitly and installs the matching
LLVM tools:

```bash
rustup component add llvm-tools-preview --toolchain <release-toolchain>
```

The production workflow pins both its macOS runner generation and Xcode
version. Every target writes a JSON receipt containing the Rust LLVM versions,
Xcode, Apple clang, host macOS, and iPhoneOS, iPhoneSimulator, and macOS SDK
versions. Receipts are written under:

```text
kotlin/anoncreds/build/reports/openssl-namespace/
anoncreds/out/openssl-namespace/
```

The receipt's replay assertion is intentionally limited to the LLVM symbol
transformation. It does not claim that two independent C/Rust source builds
are byte-for-byte reproducible.

## Validation

Run the collision fixture:

```bash
tests/openssl-namespace/run.sh
```

The fixture links a namespaced archive with a second provider that defines the
same unprefixed symbols. It first proves that the unmodified archive binds at
least one consumer to the wrong provider in both static-library orders, then
verifies that the namespaced archive reaches the correct implementation in
both orders.

Build the KMP Apple archives through their normal Cargo tasks:

```bash
cd kotlin
./gradlew \
  :anoncreds_uniffi:cargoBuildIosArm64Release \
  :anoncreds_uniffi:cargoBuildIosSimulatorArm64Release \
  :anoncreds_uniffi:cargoBuildIosX64Release \
  :anoncreds_uniffi:cargoBuildMacOSArm64Release \
  :anoncreds_uniffi:cargoBuildMacOSX64Release
```

Build the Swift XCFramework through its existing entry point:

```bash
anoncreds/build-swift-framework.sh
```

CI runs the fixture and both Apple packaging paths. Android, JVM, Linux, and
Windows build paths do not invoke the namespace pass.
