#!/bin/sh
mkdir kthg
cross build --release --target aarch64-unknown-linux-gnu && cp ./target/aarch64-unknown-linux-gnu/release/kthg ./aarch64/kthg
