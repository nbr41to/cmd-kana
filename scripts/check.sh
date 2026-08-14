#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_dir"

if [ -n "${CARGO:-}" ]; then
    cargo_bin=$CARGO
elif command -v cargo >/dev/null 2>&1; then
    cargo_bin=cargo
else
    cargo_bin="$HOME/.cargo/bin/cargo"
fi

"$cargo_bin" fmt --all -- --check
"$cargo_bin" test --locked
"$cargo_bin" clippy --all-targets --locked -- -D warnings
CARGO="$cargo_bin" ./scripts/package.sh

codesign --verify --deep --strict target/release/cmd-kana.app
plutil -lint target/release/cmd-kana.app/Contents/Info.plist
echo "Release checks passed."
