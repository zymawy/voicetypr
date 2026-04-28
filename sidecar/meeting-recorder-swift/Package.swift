// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "MeetingRecorderSidecar",
    platforms: [
        .macOS(.v13)
    ],
    targets: [
        .executableTarget(
            name: "MeetingRecorderSidecar",
            path: "Sources/MeetingRecorderSidecar"
        )
    ]
)
