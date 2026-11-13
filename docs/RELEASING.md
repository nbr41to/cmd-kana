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

### In-app update assets

The updater uses GitHub's latest **published stable Release**, not commits or tags alone. Publish a tag named `vMAJOR.MINOR.PATCH` (or `MAJOR.MINOR.PATCH`) and attach:

- `cmd-kana-MAJOR.MINOR.PATCH-macos-notarized.zip`
- `cmd-kana-MAJOR.MINOR.PATCH-macos-notarized.zip.sha256`

These are the existing `scripts/notarize.sh` outputs. The updater also recognizes the corresponding `-macos.zip` and `.zip.sha256` names from `scripts/archive.sh` for internal testing; keep public builds signed and notarized as described above. The notarized ZIP takes precedence when both exist. Each ZIP must be at most 100 MiB and contain only the `cmd-kana.app` tree, without AppleDouble metadata. The app uses GitHub's SHA-256 asset digest when available and otherwise downloads the matching checksum file.

The bundled version must exactly match the tag, without its optional `v` prefix. The app must retain its bundle identifier and satisfy the installed app's designated signing requirement. Each replacement must include the updater's `--update-complete` startup acknowledgment, which is sent automatically once Cocoa finishes launching.

Upload assets to a draft Release first, then publish it after verification so users do not see a partially uploaded update. The current release-candidate workflow does not publish a Release automatically. A repository with no published Release is reported as having no distribution available, not as being up to date.

## Rollback

If a regression is found, mark the release as affected, restore the previous notarized artifact, and issue a patch version. Do not reuse or move an existing version tag.
