#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
SOURCE_ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)
TEST_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/anoncreds-openssl-namespace.XXXXXX")
trap 'rm -rf "$TEST_ROOT"' EXIT

CLANG=$(xcrun --find clang)
AR=$(xcrun --find ar)
MACOS_SDK=$(xcrun --sdk macosx --show-sdk-path)

"$CLANG" -c "$SCRIPT_DIR/vendor_crypto.c" -o "$TEST_ROOT/vendor_crypto.o"
"$CLANG" -c "$SCRIPT_DIR/vendor_ssl.c" -o "$TEST_ROOT/vendor_ssl.o"
"$CLANG" -c "$SCRIPT_DIR/anoncreds_consumer.c" -o "$TEST_ROOT/anoncreds_consumer.o"
"$CLANG" -c "$SCRIPT_DIR/other_provider.c" -o "$TEST_ROOT/other_provider.o"
"$CLANG" -c "$SCRIPT_DIR/main.c" -o "$TEST_ROOT/main.o"

"$AR" rcs "$TEST_ROOT/libcrypto.a" "$TEST_ROOT/vendor_crypto.o"
"$AR" rcs "$TEST_ROOT/libssl.a" "$TEST_ROOT/vendor_ssl.o"
"$AR" rcs "$TEST_ROOT/libanoncreds_uniffi.a" \
  "$TEST_ROOT/anoncreds_consumer.o" \
  "$TEST_ROOT/vendor_crypto.o" \
  "$TEST_ROOT/vendor_ssl.o"
"$AR" rcs "$TEST_ROOT/libother_provider.a" "$TEST_ROOT/other_provider.o"

# Prove that the fixture exercises the failure mode before applying the patch.
# Both orders link, but at least one consumer binds to the other provider and
# therefore returns the wrong value.
"$CLANG" -isysroot "$MACOS_SDK" "$TEST_ROOT/main.o" \
  "$TEST_ROOT/libanoncreds_uniffi.a" \
  "$TEST_ROOT/libother_provider.a" \
  -o "$TEST_ROOT/baseline-anoncreds-first"
if "$TEST_ROOT/baseline-anoncreds-first"; then
  echo "Baseline fixture unexpectedly isolated the anoncreds-first order." >&2
  exit 1
fi
"$CLANG" -isysroot "$MACOS_SDK" "$TEST_ROOT/main.o" \
  "$TEST_ROOT/libother_provider.a" \
  "$TEST_ROOT/libanoncreds_uniffi.a" \
  -o "$TEST_ROOT/baseline-provider-first"
if "$TEST_ROOT/baseline-provider-first"; then
  echo "Baseline fixture unexpectedly isolated the provider-first order." >&2
  exit 1
fi

ANONCREDS_NAMESPACE_TESTING=1 \
  python3 "$SOURCE_ROOT/scripts/namespace_anoncreds_openssl.py" \
    --archive "$TEST_ROOT/libanoncreds_uniffi.a" \
    --libcrypto "$TEST_ROOT/libcrypto.a" \
    --libssl "$TEST_ROOT/libssl.a" \
    --target regression-fixture \
    --receipt "$TEST_ROOT/receipt.json" \
    --minimum-symbol-count 2 \
    --skip-uniffi-check

"$CLANG" -isysroot "$MACOS_SDK" "$TEST_ROOT/main.o" \
  "$TEST_ROOT/libanoncreds_uniffi.a" \
  "$TEST_ROOT/libother_provider.a" \
  -o "$TEST_ROOT/anoncreds-first"
"$TEST_ROOT/anoncreds-first"

"$CLANG" -isysroot "$MACOS_SDK" "$TEST_ROOT/main.o" \
  "$TEST_ROOT/libother_provider.a" \
  "$TEST_ROOT/libanoncreds_uniffi.a" \
  -o "$TEST_ROOT/provider-first"
"$TEST_ROOT/provider-first"

echo "OpenSSL namespace link-order fixture passed in both orders."
