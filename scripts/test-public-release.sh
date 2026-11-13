#!/bin/sh
# Verify the free distribution, and reject false notarization and corruption.
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
version=$(python3 "$project_dir/scripts/check-release.py" --version-only)
temporary=$(mktemp -d "${TMPDIR:-/tmp}/cmd-kana-public-test.XXXXXX")
trap 'rm -rf "$temporary"' EXIT HUP INT TERM
name="cmd-kana-$version-macos-notarized.zip"
"$project_dir/scripts/verify-public-release.sh" "$project_dir/target/dist/cmd-kana-$version-macos.zip"
cp "$project_dir/target/dist/cmd-kana-$version-macos.zip" "$temporary/$name"
(cd "$temporary" && shasum -a 256 "$name" > "$name.sha256")
if "$project_dir/scripts/verify-public-release.sh" --notarized "$temporary/$name" > "$temporary/result.log" 2>&1; then
    echo "An ad-hoc signed development ZIP passed the public release gate." >&2
    exit 1
fi
if ! grep -q 'A Developer ID Application signature is required' "$temporary/result.log"; then
    cat "$temporary/result.log" >&2
    echo "Public release verification failed before the expected signature check." >&2
    exit 1
fi
name="cmd-kana-$version-macos.zip"
cp "$project_dir/target/dist/$name" "$temporary/$name"
(cd "$temporary" && shasum -a 256 "$name" > "$name.sha256")
printf 'corruption' >> "$temporary/$name"
if "$project_dir/scripts/verify-public-release.sh" "$temporary/$name" > "$temporary/result.log" 2>&1; then
    echo "A corrupted ZIP passed distribution verification." >&2
    exit 1
fi
echo "Free distribution verified; false notarization and checksum corruption correctly rejected."
