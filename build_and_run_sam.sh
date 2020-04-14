#!/bin/sh
cd $HOME/rust/rust_aws_sam_local/
docker run --rm -it -v "$(pwd)":/home/rust/src \
-v "$(pwd)/cargo-git":/home/rust/.cargo/git \
-v "$(pwd)/cargo-registry":/home/rust/.cargo/registry \
-v "$(pwd)/target":/home/rust/src/target \
ekidd/rust-musl-builder cargo build --release
cd $HOME/rust/rust_aws_sam_local/sam
zip -j rust.zip ../target/x86_64-unknown-linux-musl/release/bootstrap
sam local invoke -e test.json
