# Security policy

## Supported versions

Only the latest released version receives security fixes.

## Reporting

Do not include passwords, private text, crash dumps containing personal data, or other secrets in a public issue. Report the minimum reproducible behavior and the affected macOS and cmd-kana versions to the project maintainer through a private channel when one is available.

## Privacy model

cmd-kana requires Input Monitoring and Accessibility permissions to observe Command/key events and replace the immediately preceding text. It does not upload input or persist typed key history. Recorded physical keys are held in memory only and cleared at input boundaries such as spaces, newlines, mouse clicks, and shortcuts.

## Updates

The "Download source (GitHub)" action opens `https://github.com/nbr41to/cmd-kana/releases/latest` in the user's default browser. Browser requests use that browser's settings/session; the app does not attach input history or credentials to the URL.

Official free distributions are ad-hoc signed and are not Apple-notarized or Developer ID-signed. macOS may require a per-app first-launch exception in Privacy & Security. This distribution does not disable Gatekeeper globally or remove quarantine automatically. Download through this repository's GitHub Releases; an ad-hoc identifier and a GitHub checksum do not constitute an Apple-verified publisher identity.

Network access occurs only after the user chooses to check for updates. The app requests the latest stable release from `api.github.com/repos/nbr41to/cmd-kana/releases/latest`. If the user accepts an update, it downloads the matching app ZIP and, when needed, its SHA-256 sidecar from GitHub Releases and GitHub's asset CDN. Requests use an ephemeral session without stored cookies or credentials. GitHub receives normal connection information, including the user's IP address; no input history is attached.

The updater accepts stable `vMAJOR.MINOR.PATCH` or `MAJOR.MINOR.PATCH` tags and never downgrades. Archives must have the expected repository URL and filename. The app checks the archive size and SHA-256 (GitHub's asset digest or the matching `.sha256` file), restricts extraction to `cmd-kana.app`, and rejects path traversal, external symlinks, hardlinks, and oversized archives. Before staging the replacement, it checks the bundle identifier, release version, executable architecture, signature validity, and the installed app's designated code requirement. The hash is supplied by GitHub; it is an integrity check rather than an independent publisher signature. Ad-hoc builds use the repository's explicit identifier requirement; Developer ID builds additionally retain their signing identity requirement.

Installation runs as the current user and requires a writable app directory. A separate copy of the installed executable waits for the original process to exit, backs up the old bundle, and installs the verified replacement. The backup is removed only after the replacement signals that Cocoa has finished launching. Installation or startup failures restore the old bundle where filesystem access permits; a failed restoration reports the backup path for manual recovery.
