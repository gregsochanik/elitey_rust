// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "EliteGUI",
    platforms: [.macOS(.v13)],
    targets: [
        .target(
            name: "elite_ffiFFI",
            path: "Sources/elite_ffiFFI",
            publicHeadersPath: "include"
        ),
        .executableTarget(
            name: "EliteGUI",
            dependencies: ["elite_ffiFFI"],
            path: "Sources/EliteGUI",
            linkerSettings: [.unsafeFlags(["../target/release/libelite_ffi.a"])]
        ),
    ]
)
