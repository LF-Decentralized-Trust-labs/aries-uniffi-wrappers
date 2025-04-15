#!/bin/sh

set -eo pipefail

pushd `dirname $0`
trap popd EXIT

NAME="anoncreds_uniffi"
BUNDLE_IDENTIFIER="org.hyperledger.$NAME"
LIBRARY_NAME="lib$NAME.a"
OUT_PATH="out/kmpp-uniffi"
WRAPPER_PATH="../Sources/Anoncreds"
AARCH64_APPLE_DARWIN_PATH="./target/aarch64-apple-darwin/release"
X86_64_APPLE_DARWIN_PATH="./target/x86_64-apple-darwin/release"

apple_targets=("aarch64-apple-ios" "aarch64-apple-ios-sim" "x86_64-apple-ios" "aarch64-apple-darwin" "x86_64-apple-darwin")

windows_targets=("x86_64-pc-windows-gnu")
windows_jna=("win32-x86-64")

linux_targets=("x86_64-unknown-linux-gnu")
linux_jna=("linux-x86-64")

android_targets=("aarch64-linux-android" "armv7-linux-androideabi" "i686-linux-android" "x86_64-linux-android")
android_jni=("arm64-v8a" "armeabi-v7a" "x86" "x86_64")

# Build for apple targets
for target in "${apple_targets[@]}"; do
  echo "Building for $target..."
  rustup target add $target
  cargo build --release --target $target
done


# Merge mac libraries with lipo
mkdir -p $OUT_PATH/macos-native/static
mkdir -p $OUT_PATH/jna/darwin

# .a for Native
lipo -create $AARCH64_APPLE_DARWIN_PATH/lib$NAME.a \
             $X86_64_APPLE_DARWIN_PATH/lib$NAME.a \
     -output $OUT_PATH/macos-native/static/lib$NAME.a
     
# dylib for JVM
lipo -create $AARCH64_APPLE_DARWIN_PATH/lib$NAME.dylib \
             $X86_64_APPLE_DARWIN_PATH/lib$NAME.dylib \
     -output $OUT_PATH/jna/darwin/lib$NAME.dylib  

# Build for JVM windows desktop environments
for key in "${!windows_targets[@]}"; do
  target=${windows_targets[$key]}
  jnaDir=$OUT_PATH/jna/${windows_jna[$key]}
  mkdir -p $jnaDir
  echo "Building for $target..."
  rustup target add $target
  cargo build --release --target $target
  cp ./target/$target/release/$NAME.dll $jnaDir/$NAME.dll
done

cargo install cross --git https://github.com/cross-rs/cross

# Build for JVM linux desktop environments
for key in "${!linux_targets[@]}"; do
  target=${linux_targets[$key]}
  jnaDir=$OUT_PATH/jna/${linux_jna[$key]}
  mkdir -p $jnaDir
  echo "Building for $target..."
  rustup target add $target
  cross build --release --target $target
  cp ./target/$target/release/lib$NAME.so $jnaDir/lib$NAME.so
done

# Build for android targets
for target in "${android_targets[@]}"; do
  echo "Building for $target..."
  cross build --release --target $target
done

# Create JNI Libs folder
for key in "${!android_targets[@]}"; do
  mkdir -p $OUT_PATH/jniLibs/${android_jni[$key]}
  cp ./target/${android_targets[$key]}/release/lib$NAME.so $OUT_PATH/jniLibs/${android_jni[$key]}/lib$NAME.so || echo ""
  echo "${android_targets[$key]}: ${android_jni[$key]}"
done

# Generate wrapper
echo "Generating wrapper..."
mkdir -p $OUT_PATH
cargo install --bin uniffi-bindgen-kotlin-multiplatform uniffi_bindgen_kotlin_multiplatform@0.1.0
CURRENT_ARCH=$(rustc --version --verbose | grep host | cut -f2 -d' ')
uniffi-bindgen-kotlin-multiplatform --lib-file ./target/$CURRENT_ARCH/release/$LIBRARY_NAME --out-dir $OUT_PATH uniffi/anoncreds_uniffi.udl

cargo install --bin uniffi-bindgen-kotlin-multiplatform uniffi_bindgen_kotlin_multiplatform@0.1.0

cargo run --bin uniffi-bindgen generate --library target/aarch64-linux-android/release/libanoncreds_uniffi.so   --language kotlin --out-dir out
 uniffi-bindgen-kotlin-multiplatform --lib-file target/aarch64-linux-android/release/libanoncreds_uniffi.so --out-dir out uniffi/anoncreds_uniffi.udl