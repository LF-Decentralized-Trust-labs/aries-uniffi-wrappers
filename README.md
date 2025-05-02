# Aries uniffi wrappers

This repository contains the wrappers for the following libraries:
- Aries Askar: https://github.com/hyperledger/aries-askar
- AnonCreds Rust: https://github.com/hyperledger/anoncreds-rs
- Indy VDR: https://github.com/hyperledger/indy-vdr

The wrappers are generated using [UniFFI](https://github.com/mozilla/uniffi-rs).
UniFFI can generate several language bindings including Swift, Kotlin, and Python.
Kotlin wrappers are generated with [Gobley](https://gobley.dev/).

## Usage

### Swift

The Swift wrappers are distributed as a Swift Package.
To use the wrappers in your project, add the following dependency to your `Package.swift`:
```swift
    .package(url: "https://github.com/hyperledger/aries-uniffi-wrappers", from: "0.1.0"),
```

And add the following dependency to your target:
```swift
    .target(
        name: "MyTarget",
        dependencies: [
            .product(name: "AriesAskar", package: "aries-uniffi-wrappers"),
            .product(name: "AnonCreds", package: "aries-uniffi-wrappers"),
            .product(name: "IndyVdr", package: "aries-uniffi-wrappers"),
        ]
    ),
```

Take a look at the tests in `swift/Tests` directory for examples on how to use the wrappers.

### Kotlin

The Kotlin wrappers are distributed as a Maven package hosted by GitHub Packages.
To add a github packages repository in a seperate project you will have to have a github token with `read:packages` permissions. Then you will add the repository to your `build.gradle.kts` like so:
```kotlin
    repositories {
        mavenCentral()
        google()
        maven {
            setUrl("https://maven.pkg.github.com/hyperledger/aries-uniffi-wrappers")
            credentials {
                username = getExtraString("githubUsername")
                password = getExtraString("githubToken")
            }
        }
    }
```

Now you will add the libraries as a dependency in your `build.gradle.kts` like so:
```kotlin
    dependencies {
        implementation("org.hyperledger:anoncreds_uniffi:0.1.1-wrapper.1")
        implementation("org.hyperledger:indy_vdr_uniffi:0.1.1-wrapper.1")
        implementation("org.hyperledger:askar_uniffi:0.1.1-wrapper.1")
    }
```

Take a look at the tests in `kotlin/${library}/src/commonTest` for usage examples.

## Contributing

Pull requests are welcome! We enforce [developer certificate of origin](https://developercertificate.org/) (DCO) commit signing. See guidance [here](https://github.com/apps/dco).

Please see our [Developer Guide](DEVELOP.md) for more information.

## License

Aries uniffi wrappers are licensed under the [Apache License 2.0](LICENSE).
