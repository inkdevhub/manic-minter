#!/bin/bash

# set -eux

cargo contract build --manifest-path manicminter/Cargo.toml --release
cargo contract build --manifest-path oxygen/Cargo.toml --release
