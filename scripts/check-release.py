#!/usr/bin/env python3
"""Check release metadata without Apple credentials or a macOS SDK."""

import argparse
import json
from pathlib import Path
import plistlib
import re
import sys


def validate(root):
    cargo = (root / "Cargo.toml").read_text()
    match = re.search(r'^version = "(\d+\.\d+\.\d+)"$', cargo, re.MULTILINE)
    if not match:
        raise ValueError("Cargo.toml must contain a stable MAJOR.MINOR.PATCH version")
    version = match.group(1)
    info = plistlib.loads((root / "app/Info.plist").read_bytes())
    if info.get("CFBundleShortVersionString") != version:
        raise ValueError("Cargo.toml and Info.plist versions differ")
    if not str(info.get("CFBundleVersion", "")).isdigit():
        raise ValueError("CFBundleVersion must be numeric")
    lock = (root / "Cargo.lock").read_text()
    if not re.search(r'name = "cmd-kana"\nversion = "' + re.escape(version) + '"', lock):
        raise ValueError("Cargo.lock has not been updated")
    changelog = (root / "CHANGELOG.md").read_text()
    if not re.search(r'^## \[' + re.escape(version) + r'\] - \d{4}-\d{2}-\d{2}$', changelog, re.MULTILINE):
        raise ValueError("CHANGELOG.md has no dated entry for this version")
    for suffix in (".md", "-checklist.md"):
        if not (root / "docs/releases" / ("v" + version + suffix)).is_file():
            raise ValueError("Release notes or validation checklist are missing")
    plan = json.loads((root / ".github/release.json").read_text())
    if plan.get("version") != version:
        raise ValueError("Release publication plan version differs from the app")
    if plan.get("distribution") != "unnotarized" or type(plan.get("publish")) is not bool:
        raise ValueError("Release plan must explicitly select unnotarized distribution and a boolean publish flag")
    return version


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version-only", action="store_true")
    args = parser.parse_args()
    try:
        version = validate(Path(__file__).resolve().parent.parent)
    except (OSError, ValueError, plistlib.InvalidFileException) as error:
        print(f"Release metadata error: {error}", file=sys.stderr)
        return 1
    print(version if args.version_only else f"Release metadata is consistent: v{version}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
