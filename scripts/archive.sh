#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
app="$project_dir/target/release/cmd-kana.app"
dist="$project_dir/target/dist"

"$project_dir/scripts/package.sh"
version=$(plutil -extract CFBundleShortVersionString raw "$app/Contents/Info.plist")
archive="$dist/cmd-kana-$version-macos.zip"
checksum="$archive.sha256"

mkdir -p "$dist"
rm -f "$archive" "$checksum"
COPYFILE_DISABLE=1 ditto -c -k --norsrc --keepParent "$app" "$archive"
shasum -a 256 "$archive" > "$checksum"
echo "$archive"
