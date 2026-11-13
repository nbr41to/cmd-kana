# Releasing

## Distribution policy

cmd-kana is distributed for free through GitHub Releases **without Apple Developer Program membership**. The normal `-macos.zip` contains an ad-hoc-signed app, not a Developer ID-signed or Apple-notarized app. The signature permits integrity checks but does not establish an Apple-verified publisher identity. macOS can block its first launch until the user explicitly allows this app in Privacy & Security.

The current release version/build is `0.10.0` / `24`. See [release notes](releases/v0.10.0.md) and [validation records](releases/v0.10.0-checklist.md). Keep unperformed interactive checks marked as unverified; CI success does not establish behavior in every IME or focused application.

## Publish from main

The publication request is explicit in `.github/release.json`:

```json
{
  "version": "0.10.0",
  "distribution": "unnotarized",
  "publish": true
}
```

After a successful **push CI run on this repository's current main commit**, [Release unnotarized app](https://github.com/nbr41to/cmd-kana/actions/workflows/release.yml) reads this plan. A manual Run workflow on `main` is also supported. Fork/PR CI runs do not trigger publication.

The repository must be **Public**. A published Release inside a Private repository is visible only to authorized users, and the app's unauthenticated update requests cannot read it. The workflow checks visibility and skips publication with an explicit pending notice while the repository is Private. Only the owner should change visibility after reviewing the source and history that will become public.

The workflow:

1. Checks matching package, lockfile, bundle, changelog, release documents, and publication-plan versions.
2. Leaves an already published version unchanged and refuses to move an existing tag. An interrupted draft can be resumed only if it belongs to the exact same source commit.
3. Runs the full macOS release checks and metadata regression tests on an Apple Silicon runner with a macOS 11.0 deployment target.
4. Builds `cmd-kana-VERSION-macos.zip` and its `.zip.sha256` file using the ad-hoc signing identity. No Apple certificate or notarization secrets are used.
5. Verifies the actual ZIP after extraction: checksum, bundled license notices, app version/build/identifier, architecture, executable minimum OS, and code-signature integrity. Regression tests also reject corrupted ZIPs and ad-hoc ZIPs falsely labeled as notarized.
6. Creates a draft Release, uploads both verified assets, verifies their presence, then publishes it as the latest stable Release. The tag targets the source commit that was checked out and tested.

Set `publish` to `false` to prepare changes without publishing a new version. Existing public releases are immutable to this workflow: it never replaces their assets or moves their tags. Later commits with the same version are a no-op; issue a new version for fixes.

## Prepare the next version

1. Choose an unused stable version such as `0.10.1`.
2. Update `Cargo.toml`, `Cargo.lock`, and `CFBundleShortVersionString` in `app/Info.plist`; increment `CFBundleVersion`.
3. Move the relevant `CHANGELOG.md` entries into a dated section for that version.
4. Add `docs/releases/vVERSION.md` and `vVERSION-checklist.md`. Release notes must disclose the lack of Apple notarization and explain the per-app first-launch exception.
5. Update `.github/release.json` to that version. Use `publish: false` while preparing; set it to `true` when requesting publication.
6. Run `python3 scripts/check-release.py`, `./scripts/check.sh` on a Mac, and the relevant checks in [TESTING.md](TESTING.md). Record actual manual results and known limitations.
7. Push to `main`. CI must succeed before automatic publication starts.

## Build and inspect locally

On an Apple Silicon Mac with Rust, Xcode Command Line Tools, and Python 3:

```sh
./scripts/check.sh
./scripts/archive.sh
./scripts/verify-public-release.sh
./scripts/test-public-release.sh
```

`verify-public-release.sh` validates the unnotarized distribution by default. It intentionally does not require `spctl` acceptance or an Apple notarization ticket for this route. It does not alter Gatekeeper, remove quarantine, or change system security settings.

Checksums use relative filenames, so downloads can be checked from the containing directory:

```sh
shasum -a 256 -c cmd-kana-0.10.0-macos.zip.sha256
```

ZIPs must be at most 100 MiB and contain only the `cmd-kana.app` tree without AppleDouble metadata. Include `LICENSE.txt` and `ThirdPartyNotices.txt` in the app resources. Keep the bundle identifier and designated signing requirement stable across updates, and retain the `--update-complete` startup acknowledgment.

## Optional Apple notarization later

Developer ID signing and Apple notarization are optional and normally require a paid Apple Developer Program membership. They are not prerequisites for the current distribution policy. If a maintainer later chooses this route, configure a Developer ID Application certificate and a `notarytool` keychain profile on that Mac:

```sh
export CODESIGN_IDENTITY='Developer ID Application: Your Name (TEAMID)'
export NOTARY_PROFILE='cmd-kana-notary'
./scripts/notarize.sh
./scripts/verify-public-release.sh --notarized
```

This produces the separately named `-macos-notarized.zip` and checksum, requires a valid Developer ID signature, hardened runtime, stapled Apple ticket, and Gatekeeper acceptance. Never label an ad-hoc build as notarized. Introduce any signing-policy change in a new release version and document possible permission migration.

The in-app updater accepts both filename variants, prefers a notarized ZIP when present, checks GitHub's asset SHA-256 (or the matching sidecar), and never downgrades. A repository commit or tag alone is not a downloadable update; a published stable Release with matching assets is required.

## Rollback

Keep prior Releases available. If a regression is reported, document the affected version and issue a fixed version. Do not move an existing tag or replace a published ZIP under the same version. Preserve actual validation evidence rather than marking unperformed interactive checks as passed.
