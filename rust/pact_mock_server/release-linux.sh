#!/bin/bash -x

if [ "" = "$1" ]; then
  echo "Usage: "
  echo "  ./release-linux.sh version"
  exit 1
fi

cargo clean
cargo build --release
gzip -c ../target/release/libpact_mock_server.so > ../target/release/libpact_mock_server-linux-x86_64-$1.so.gz
gzip -c ../target/release/libpact_mock_server.a > ../target/release/libpact_mock_server-linux-x86_64-$1.a.gz
