# Releasing

## 1. Prepare

1. Move completed entries from `Unreleased` in `CHANGELOG.md` to the new version.
2. Keep the versions in `Cargo.toml` and `app/Info.plist` identical.
3. Increment `CFBundleVersion` in `app/Info.plist`.
4. Run `./scripts/check.sh` and complete the manual matrix in `docs/TESTING.md`.

## 2. Create an internal release candidate

```sh
./scripts/archive.sh
```

This produces an ad-hoc-signed ZIP and SHA-256 checksum under `target/dist`. It is for testing, not the preferred public download.

Pushing a matching tag such as `v0.7.1` runs the release-candidate workflow. The workflow deliberately does not publish a GitHub Release, preventing accidental distribution of an unnotarized build.

## 3. Sign and notarize a public build

Create a notarytool keychain profile once, then run:

```sh
export CODESIGN_IDENTITY='Developer ID Application: Example (TEAMID)'
export NOTARY_PROFILE='cmd-kana-notary'
./scripts/notarize.sh
```

The script signs with the hardened runtime, submits the app to Apple, staples and validates the ticket, assesses it with Gatekeeper, and creates a final ZIP plus checksum.

## 4. Publish

1. Verify the checksum on a second machine.
2. Install and run the notarized ZIP in a clean macOS account.
3. Create release notes from `CHANGELOG.md`, including known limitations and the manual test matrix.
4. Upload only the `-notarized.zip` and its `.sha256` file.
5. Keep the previous release available for rollback.

## Rollback

If a regression is found, mark the release as affected, restore the previous notarized artifact, and issue a patch version. Do not reuse or move an existing version tag.
