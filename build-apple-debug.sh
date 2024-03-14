#! /bin/sh

echo "Setting up the rust environment..."
rustup target add aarch64-apple-ios
cargo install cbindgen

echo "Building target aarch64-apple-ios..."
cargo build --target aarch64-apple-ios

echo "Generating includes..."
mkdir -p target/include/
rm -rf target/include/*
cbindgen --config cbindgen.toml -l C -o target/include/dns2socks.h
cat > target/include/module.modulemap <<EOF
framework module dns2socks {
    umbrella header "dns2socks.h"

    export *
    module * { export * }
}
EOF

echo "Creating XCFramework"
rm -rf ./dns2socks.xcframework
xcodebuild -create-xcframework \
    -library ./target/aarch64-apple-ios/debug/libdns2socks.a -headers ./target/include/ \
    -output ./dns2socks.xcframework
