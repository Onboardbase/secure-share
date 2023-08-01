#!/bin/bash
rm -rf dist
rm -rf target/x86_64-unknown-linux-gnu
cargo build --release --locked --target x86_64-unknown-linux-gnu
mkdir dist
strip "target/x86_64-unknown-linux-gnu/release/scs"
cp "target/x86_64-unknown-linux-gnu/release/scs" "dist/"
rm -rf releases
mkdir releases
tar -czf releases/scs.tar.gz -C dist scs
chmod a+rx releases/scs.tar.gz
