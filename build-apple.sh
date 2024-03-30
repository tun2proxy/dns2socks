#! /bin/sh

echo "Setting up the rust environment..."
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios x86_64-apple-darwin aarch64-apple-darwin
cargo install cbindgen

echo "Building..."

echo "cargo build --release --target x86_64-apple-darwin"
cargo build --release --target x86_64-apple-darwin

echo "cargo build --release --target aarch64-apple-darwin"
cargo build --release --target aarch64-apple-darwin

echo "cargo build --release --target aarch64-apple-ios"
cargo build --release --target aarch64-apple-ios

echo "cargo build --release --target x86_64-apple-ios"
cargo build --release --target x86_64-apple-ios

echo "cargo build --release --target aarch64-apple-ios-sim"
cargo build --release --target aarch64-apple-ios-sim

echo "Generating includes..."
mkdir -p target/include/
rm -rf target/include/*
cbindgen --config cbindgen.toml -l C -o target/include/dns2socks.h
cat > target/include/dns2socks.modulemap <<EOF
framework module dns2socks {
    umbrella header "dns2socks.h"

    export *
    module * { export * }
}
EOF

echo "lipo..."
echo "Simulator"
lipo -create \
    target/aarch64-apple-ios-sim/release/libdns2socks.a \
    target/x86_64-apple-ios/release/libdns2socks.a \
    -output ./target/libdns2socks-ios-sim.a

echo "MacOS"
lipo -create \
    target/aarch64-apple-darwin/release/libdns2socks.a \
    target/x86_64-apple-darwin/release/libdns2socks.a \
    -output ./target/libdns2socks-macos.a

echo "Creating XCFramework"
rm -rf ./dns2socks.xcframework
xcodebuild -create-xcframework \
    -library ./target/aarch64-apple-ios/release/libdns2socks.a -headers ./target/include/ \
    -library ./target/libdns2socks-ios-sim.a -headers ./target/include/ \
    -library ./target/libdns2socks-macos.a -headers ./target/include/ \
    -output ./dns2socks.xcframework
