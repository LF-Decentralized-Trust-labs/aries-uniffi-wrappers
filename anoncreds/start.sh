#!/bin/bash
set -e

echo "🧹 Limpando build anterior..."
cargo clean

echo "🔧 Compilando biblioteca nativa (cdylib)..."
cross build --release --target aarch64-linux-android

echo "Gerando .kt"
cargo run --bin uniffi-bindgen generate --library target/aarch64-linux-android/release/libanoncreds_uniffi.so   --language kotlin --out-dir out

echo "✅ Build finalizado!"