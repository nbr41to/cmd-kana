# Testing

## Automated checks

Run the complete local gate on macOS:

```sh
./scripts/check.sh
```

It verifies formatting, unit tests, Clippy warnings, release compilation, plist validity, version consistency, and the app signature.

The pure Rust tests cover:

- left/right Command single taps;
- left double-tap timing boundaries and exact input replay;
- right double taps remaining Kana-only;
- shortcut and simultaneous-Command suppression;
- history boundaries, deletion, and the 128-key limit;
- US-keyboard keycode-to-romaji output;
- common `n`, gemination, compound-kana, and unsupported-input cases.
- mixed full-width Latin and Kana output such as `kfa → ｋふぁ`.

## Manual release matrix

Automated tests cannot reproduce macOS TCC permissions, every focused text control, or a real IME. Before release, test the notarized build in a clean macOS user account.

| Area | Required check |
| --- | --- |
| Permissions | Fresh launch reports both permissions accurately and opens each Settings pane. |
| Permission recovery | Launch without permission, grant both permissions while the app remains open, and verify switching starts without restarting. |
| Single tap | Left Command selects Google Japanese Roman; right Command selects Google Japanese Kana. |
| Shortcuts | Command-C/V/Z and application shortcuts remain unchanged. |
| Double tap | `あか` becomes `aka`; `こんにちは` becomes its recorded romaji without visible cursor rewind in supported fields. |
| Controls | Verify TextEdit, Terminal, Safari, Chrome, and at least one Electron editor. |
| Settings | Window opens on launch and reopen; menu-bar visibility persists; Update works while the menu-bar item is hidden. |
| Login item | On macOS 13+, enabling and disabling launch at login matches System Settings; a denied item reports that approval is required; login launch stays in the menu bar without opening Settings. |
| Upgrade | Updating over the previous release preserves macOS permissions and settings. |
| Lifecycle | Menu Update restarts once; Quit exits; no duplicate processes remain. |
| Duplicate launch | Start the app repeatedly and overlap login/manual launch; exactly one process and menu-bar item remain. |

Record the macOS version, architecture, IME/version, tested apps, and result in the release notes.
