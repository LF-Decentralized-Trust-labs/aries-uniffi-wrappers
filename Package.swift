// swift-tools-version: 5.7
import PackageDescription
import class Foundation.ProcessInfo

var package = Package(
    name: "aries-uniffi-wrappers",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v15),
    ],
    products: [
        .library(
            name: "Anoncreds",
            targets: ["Anoncreds"]),
        .library(
            name: "Askar",
            targets: ["Askar"]),
        .library(
            name: "IndyVdr",
            targets: ["IndyVdr"]),
    ],
    dependencies: [
    ],
    targets: [
        .target(
            name: "Anoncreds",
            path: "swift/Sources/Anoncreds"),
        .testTarget(
            name: "AnoncredsTests",
            dependencies: ["Anoncreds"],
            path: "swift/Tests/AnoncredsTests"),
        .binaryTarget(
            name: "anoncreds_uniffiFFI",
            url: "https://github.com/hyperledger/aries-uniffi-wrappers/releases/download/0.3.1-binary/anoncreds_uniffiFFI.xcframework.zip",
            checksum: "53144f44014a6a31fef6fba0e263d44067373b2b2aad57e77405db2f4d3d629d"),
        .target(
            name: "Askar",
            path: "swift/Sources/Askar"),
        .testTarget(
            name: "AskarTests",
            dependencies: ["Askar"],
            path: "swift/Tests/AskarTests",
            resources: [
                .copy("resources/indy_wallet_sqlite.db")
            ]),
        .binaryTarget(
            name: "askar_uniffiFFI",
            url: "https://github.com/hyperledger/aries-uniffi-wrappers/releases/download/0.3.1-binary/askar_uniffiFFI.xcframework.zip",
            checksum: "34f53f4908f10825245a5958d71988537183b238c039955850b5ceb033710af6"),
        .target(
            name: "IndyVdr",
            path: "swift/Sources/IndyVdr"),
        .testTarget(
            name: "IndyVdrTests",
            dependencies: ["IndyVdr"],
            path: "swift/Tests/IndyVdrTests",
            resources: [
                .copy("resources/genesis_sov_buildernet.txn")
            ]),
        .binaryTarget(
            name: "indy_vdr_uniffiFFI",
            url: "https://github.com/hyperledger/aries-uniffi-wrappers/releases/download/0.3.1-binary/indy_vdr_uniffiFFI.xcframework.zip",
            checksum: "a0640f9d13f218771ff546d6a6e0c28747a674c42f02ba119703b1bc6fb08e72")
    ]
)

let anoncredsTarget = package.targets.first(where: { $0.name == "Anoncreds" })
let askarTarget = package.targets.first(where: { $0.name == "Askar" })
let indyVdrTarget = package.targets.first(where: { $0.name == "IndyVdr" })

if ProcessInfo.processInfo.environment["USE_LOCAL_XCFRAMEWORK"] == nil {
    anoncredsTarget?.dependencies.append("anoncreds_uniffiFFI")
    askarTarget?.dependencies.append("askar_uniffiFFI")
    indyVdrTarget?.dependencies.append("indy_vdr_uniffiFFI")
} else {
    package.targets.append(.binaryTarget(
        name: "anoncreds_uniffiFFI_local",
        path: "anoncreds/out/anoncreds_uniffiFFI.xcframework"))
    package.targets.append(.binaryTarget(
        name: "askar_uniffiFFI_local",
        path: "askar/out/askar_uniffiFFI.xcframework"))
    package.targets.append(.binaryTarget(
        name: "indy_vdr_uniffiFFI_local",
        path: "indy-vdr/out/indy_vdr_uniffiFFI.xcframework"))

    anoncredsTarget?.dependencies.append("anoncreds_uniffiFFI_local")
    askarTarget?.dependencies.append("askar_uniffiFFI_local")
    indyVdrTarget?.dependencies.append("indy_vdr_uniffiFFI_local")
}
