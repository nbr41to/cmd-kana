#!/bin/sh
set -eu

: "${CODESIGN_IDENTITY:?Set CODESIGN_IDENTITY to a Developer ID Application identity}"
: "${NOTARY_PROFILE:?Set NOTARY_PROFILE to an xcrun notarytool keychain profile}"

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
app="$project_dir/target/release/cmd-kana.app"
dist="$project_dir/target/dist"
submission="$dist/cmd-kana-notarization.zip"

"$project_dir/scripts/package.sh"
mkdir -p "$dist"
rm -f "$submission"
COPYFILE_DISABLE=1 ditto -c -k --norsrc --keepParent "$app" "$submission"
xcrun notarytool submit "$submission" --keychain-profile "$NOTARY_PROFILE" --wait
xcrun stapler staple "$app"
xcrun stapler validate "$app"
spctl --assess --type execute --verbose=4 "$app"

version=$(plutil -extract CFBundleShortVersionString raw "$app/Contents/Info.plist")
archive="$dist/cmd-kana-$version-macos-notarized.zip"
rm -f "$archive" "$archive.sha256"
COPYFILE_DISABLE=1 ditto -c -k --norsrc --keepParent "$app" "$archive"
shasum -a 256 "$archive" > "$archive.sha256"
echo "$archive"
