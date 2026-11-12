#!/bin/sh
# Pure input logic checks; no macOS SDK or Objective-C bridge is required.
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_dir"

cargo fmt --all -- --check
mkdir -p target/input-check
rustc --edition=2021 --test src/input.rs -D warnings -o target/input-check/tests
target/input-check/tests
clippy-driver --edition=2021 --test src/input.rs -D warnings -o target/input-check/clippy
