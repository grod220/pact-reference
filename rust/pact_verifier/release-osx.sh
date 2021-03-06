#!/bin/bash -x

if [ "" = "$1" ]; then
  echo "Usage: "
  echo "  ./release-osx.sh version"
  exit 1
fi

cargo clean
cargo build --release
cargo build --release --target x86_64-apple-ios
gzip -c target/release/pact_verifier.dylib > target/release/pact_verifier-osx-x86_64-$1.dylib.gz
gzip -c target/release/pact_verifier.a > target/release/pact_verifier-osx-x86_64-$1.a.gz
cargo build --release --target x86_64-apple-ios
gzip -c target/x86_64-apple-ios/release/pact_verifier.a > target/x86_64-apple-ios/release/pact_verifier-ios-x86_64-$1.a.gz
