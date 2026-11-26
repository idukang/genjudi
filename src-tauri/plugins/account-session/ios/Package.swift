// swift-tools-version:5.9
import PackageDescription

let package = Package(
  name: "tauri-plugin-account-session",
  platforms: [
    .iOS(.v14)
  ],
  products: [
    .library(
      name: "tauri-plugin-account-session",
      type: .static,
      targets: ["tauri-plugin-account-session"]
    )
  ],
  dependencies: [
    .package(name: "Tauri", path: "../.tauri/tauri-api")
  ],
  targets: [
    .target(
      name: "tauri-plugin-account-session",
      dependencies: [
        .product(name: "Tauri", package: "Tauri")
      ],
      path: "Sources"
    )
  ]
)
