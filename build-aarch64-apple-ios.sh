#! /bin/sh

echo "Setting up the rust environment..."
rustup target add aarch64-apple-ios
cargo install cbindgen

echo "Building target aarch64-apple-ios..."
cargo build --release --target aarch64-apple-ios

echo "Generating includes..."
mkdir -p target/include/
rm -rf target/include/*
cbindgen --config cbindgen.toml -o target/include/dns2socks.h
cat > target/include/dns2socks.modulemap <<EOF
framework module dns2socks {
    umbrella header "dns2socks.h"
    export *
    module * { export * }
}
EOF

echo "Creating XCFramework"
rm -rf ./dns2socks.xcframework
xcodebuild -create-xcframework \
    -library ./target/aarch64-apple-ios/release/libdns2socks_core.a -headers ./target/include/ \
    -output ./dns2socks.xcframework
