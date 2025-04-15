#!/bin/bash
set -e

echo "🧹 Limpando build anterior..."
cargo clean

echo "🔧 Compilando biblioteca nativa..."
cross build --release --target aarch64-linux-android
#cross build --release --target aarch64-apple-darwin
#cross build --release --target x86_64-apple-darwin

echo "🔗 Gerando .kt"
cargo run --bin uniffi-bindgen generate --library target/aarch64-linux-android/release/libanoncreds_uniffi.so   --language kotlin --out-dir out/android
#cargo run --bin uniffi-bindgen generate --library target/aarch64-apple-darwin/release/libanoncreds_uniffi.dylib   --language kotlin --out-dir out/apple

echo "✅ Build finalizado!"