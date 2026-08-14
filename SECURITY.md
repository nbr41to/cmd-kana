# Security policy

## Supported versions

Only the latest released version receives security fixes.

## Reporting

Do not include passwords, private text, crash dumps containing personal data, or other secrets in a public issue. Report the minimum reproducible behavior and the affected macOS and cmd-kana versions to the project maintainer through a private channel when one is available.

## Privacy model

cmd-kana requires Input Monitoring and Accessibility permissions to observe Command/key events and replace the immediately preceding text. It does not use the network, upload input, or persist typed key history. Recorded physical keys are held in memory only and cleared at input boundaries such as spaces, newlines, mouse clicks, and shortcuts.
