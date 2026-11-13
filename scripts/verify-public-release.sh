#!/bin/sh
# Validate the exact ZIP intended for public distribution, not a development app.
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
version=$(python3 "$project_dir/scripts/check-release.py" --version-only)
archive=${1:-"$project_dir/target/dist/cmd-kana-$version-macos-notarized.zip"}
expected_name="cmd-kana-$version-macos-notarized.zip"
test "$(uname -s)" = Darwin || { echo "Public signature checks require macOS." >&2; exit 1; }
test "$(basename -- "$archive")" = "$expected_name" || { echo "Unexpected archive name." >&2; exit 1; }
test -f "$archive" && test -f "$archive.sha256"

archive_dir=$(CDPATH= cd -- "$(dirname -- "$archive")" && pwd)
(cd "$archive_dir" && shasum -a 256 -c "$expected_name.sha256")
temporary=$(mktemp -d "${TMPDIR:-/tmp}/cmd-kana-public.XXXXXX")
trap 'rm -rf "$temporary"' EXIT HUP INT TERM
ditto -x -k "$archive_dir/$expected_name" "$temporary"
app="$temporary/cmd-kana.app"
cmp "$project_dir/LICENSE" "$app/Contents/Resources/LICENSE.txt"
cmp "$project_dir/app/vendor/libarchive/NOTICE.txt" "$app/Contents/Resources/ThirdPartyNotices.txt"
test "$(plutil -extract CFBundleShortVersionString raw "$app/Contents/Info.plist")" = "$version"
test "$(plutil -extract CFBundleIdentifier raw "$app/Contents/Info.plist")" = local.cmd-kana
test "$(plutil -extract CFBundleVersion raw "$app/Contents/Info.plist")" = \
    "$(plutil -extract CFBundleVersion raw "$project_dir/app/Info.plist")"
test "$(plutil -extract LSMinimumSystemVersion raw "$app/Contents/Info.plist")" = \
    "$(plutil -extract LSMinimumSystemVersion raw "$project_dir/app/Info.plist")"
lipo -verify_arch arm64 "$app/Contents/MacOS/cmd-kana"
xcrun vtool -show-build "$app/Contents/MacOS/cmd-kana" > "$temporary/build.txt"
python3 - "$temporary/build.txt" <<'PY'
from pathlib import Path
import re
import sys
versions = re.findall(r'\bminos\s+([\d.]+)', Path(sys.argv[1]).read_text())
def normalized(value):
    return tuple((list(map(int, value.split('.'))) + [0, 0])[:3])
if not versions or any(normalized(version) > (11, 0, 0) for version in versions):
    sys.exit("The executable does not support the advertised macOS 11.0 minimum.")
PY
codesign --verify --deep --strict "$app"
signature=$(codesign -dv --verbose=4 "$app" 2>&1)
case "$signature" in
    *"Authority=Developer ID Application:"*) ;;
    *) echo "A Developer ID Application signature is required." >&2; exit 1 ;;
esac
case "$signature" in
    *"(runtime)"*) ;;
    *) echo "Hardened runtime is required." >&2; exit 1 ;;
esac
xcrun stapler validate "$app"
spctl --assess --type execute --verbose=4 "$app"
echo "Verified public release ZIP: $expected_name"
