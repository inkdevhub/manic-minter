#!/bin/bash

# set -eux

cargo contract build --manifest-path manicminter/Cargo.toml --release
cargo contract build --manifest-path token/Cargo.toml --release
