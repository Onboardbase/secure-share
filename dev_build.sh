#!/bin/bash
rm -rf dist
rm -rf target/x86_64-unknown-linux-gnu
cargo build --release --locked --target x86_64-unknown-linux-gnu
mkdir dist
strip "target/x86_64-unknown-linux-gnu/release/share"
cp "target/x86_64-unknown-linux-gnu/release/share" "dist/"
rm -rf releases
mkdir releases
tar -czf releases/share.tar.gz -C dist share
chmod a+rx releases/share.tar.gz