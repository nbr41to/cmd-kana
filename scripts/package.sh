#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
app="$project_dir/target/release/cmd-kana.app"
if [ -n "${CARGO:-}" ]; then
    cargo_bin=$CARGO
elif command -v cargo >/dev/null 2>&1; then
    cargo_bin=cargo
else
    cargo_bin="$HOME/.cargo/bin/cargo"
fi
signing_identity=${CODESIGN_IDENTITY:--}

"$cargo_bin" build --release --locked --manifest-path "$project_dir/Cargo.toml"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$project_dir/app/Info.plist" "$app/Contents/Info.plist"
cp "$project_dir/target/release/cmd-kana" "$app/Contents/MacOS/cmd-kana"
cp "$project_dir/app/assets/cmd-kana-logo.png" "$app/Contents/Resources/AppIcon.png"
cp "$project_dir/app/vendor/libarchive/NOTICE.txt" "$app/Contents/Resources/ThirdPartyNotices.txt"

if [ "$signing_identity" = "-" ]; then
    # Keep local privacy permissions valid across ad-hoc development builds.
    codesign --force --deep --sign - \
        --requirements '=designated => identifier "local.cmd-kana"' "$app"
else
    # Public releases should use a Developer ID Application identity.
    codesign --force --deep --options runtime --timestamp \
        --sign "$signing_identity" "$app"
fi
codesign --verify --deep --strict "$app"
plutil -lint "$app/Contents/Info.plist"

cargo_version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$project_dir/Cargo.toml" | head -n 1)
bundle_version=$(plutil -extract CFBundleShortVersionString raw "$app/Contents/Info.plist")
if [ "$cargo_version" != "$bundle_version" ]; then
    echo "version mismatch: Cargo.toml=$cargo_version Info.plist=$bundle_version" >&2
    exit 1
fi
