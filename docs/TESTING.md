# Testing

## Automated checks

Run the complete local gate on macOS:

```sh
./scripts/check.sh
```

It verifies formatting, unit tests, Clippy warnings, release compilation, plist validity, version consistency, and the app signature. Update tests cover numeric version comparison, SHA-256, archive containment, matching/tampered code signatures, transactional replacement, and rollback after a failed startup. The installer tests use real temporary directories, including paths with spaces and Japanese characters.

The pure Rust tests cover:

- left/right Command single taps;
- left double-tap timing boundaries and exact input replay;
- right double taps remaining Kana-only;
- shortcut and simultaneous-Command suppression;
- history boundaries, safe cancellation after deletion, and the 128-key limit;
- US-keyboard keycode-to-romaji output;
- common `n`, gemination, compound-kana, and unsupported-input cases.
- mixed full-width Latin and Kana output such as `kfa → ｋふぁ`.
- small kana (`xtsu`, `lya`), extended syllables (`thi`, `dhu`), `nn` pairs, and alphabetic symbol shortcuts;
- all overlapping Command press/release orders, Command-click, and modifier-only chords;
- English-mode input suppression and resuming Kana recording after cancellation;
- 4,968 consecutive conversion attempts on one state, with release intervals from 1–552 ms.
- Google/Mozc native conversion dispatch, input-source changes after the first tap, empty/overflowed history, and the 499/500/550/551 ms boundaries.

### Linux input checks

With Rust, rustfmt, and Clippy on `PATH`, run:

```sh
./scripts/check-input.sh
```

This compiles and executes `src/input.rs` directly, checks formatting, and runs Clippy on the input tests. It does not compile the Objective-C bridge or emulate macOS, an IME, Accessibility replacement, or synthetic key delivery. The macOS release gate and manual matrix remain required.

For the conversion debugging pass, rendered lengths were also compared with 282 alphabetic rules without carry-over from [Mozc's default romaji table](https://github.com/google/mozc/blob/master/src/data/preedit/romanji-hiragana.tsv). This checks standard table entries; custom IME rules, mode-dependent output, and live composition behavior still require manual checks.

### Live Mozc engine checks (Linux)

With `mozc-server` installed, run:

```sh
python3 scripts/check-mozc.py --report target/mozc-check.json
```

For a locally extracted package, pass `--server /path/to/mozc_server`. The script uses an isolated temporary profile, verifies the server process, and sends only synthetic fixture inputs over local IPC. It refuses to run when a legacy `~/.mozc` profile would override isolation. It checks 2,196 composition outputs, 2,196 raw-text commits, and 63 combinations of Shift behavior and input mode. No Python packages are required.

These are real Mozc engine results. The mapping from macOS Eisu events to engine commands comes from the public macOS controller; the script does not exercise macOS TIS, TCC, CoreGraphics, or a focused application. See [Google IME verification](GOOGLE_IME_TESTING.md) for findings and remaining manual checks.

## Manual release matrix

Automated tests cannot reproduce macOS TCC permissions, every focused text control, or a real IME. Before release, test the notarized build in a clean macOS user account.

| Area | Required check |
| --- | --- |
| Permissions | Fresh launch reports both permissions accurately and opens each Settings pane. |
| Permission recovery | Launch without permission, grant both permissions while the app remains open, and verify switching starts without restarting. |
| Single tap | Left Command selects Google Japanese Roman; right Command selects Google Japanese Kana. |
| Shortcuts | Command-C/V/Z and application shortcuts remain unchanged. |
| Double tap | `あか` becomes `aka`; `こんにちは` becomes its recorded romaji without visible cursor rewind in supported fields. |
| Cancellation | For non-Google input methods, after Delete or more than 128 physical keys, a left double tap leaves the text unchanged. After a boundary or a right Command switch, the next Japanese word converts normally. Google/Mozc uses its own remaining composition instead of the recorded word. |
| Modifier chords | Command-click, Shift/Control/Option with Command, and all overlapping left/right Command release orders do not switch modes; the next standalone tap still works. |
| English input | After a left Command switch, typing `kya` and double-tapping left Command leaves exactly `kya`; switching back with right Command restores Japanese conversion. |
| Controls | Verify TextEdit, Terminal, Safari, Chrome, and at least one Electron editor. |
| Settings | Window opens on launch and reopen; menu-bar visibility persists; Check for Updates works while the menu-bar item is hidden. |
| Login item | On macOS 13+, enabling and disabling launch at login matches System Settings; a denied item reports that approval is required; login launch stays in the menu bar without opening Settings. |
| Upgrade | Updating over the previous release preserves macOS permissions and settings. |
| Lifecycle | Restart restarts once; Quit exits; no duplicate processes remain. |
| Update status | Test no published Release (404), offline/timeout, API rate limit, malformed JSON, equal/older release, and a newer numeric version such as 0.10.0 after 0.9.3. Errors must not report that the installed app is up to date. |
| Update assets | Test missing ZIP/checksum, invalid SHA-256, wrong bundle ID/version, wrong architecture, invalid signature, and a different signing identity. The running app must remain unchanged. |
| Update installation | From a writable directory, accept an update and verify one replacement process, preserved settings/permissions, a startup acknowledgment, and removal of the backup and temporary files. Repeat with a hidden menu-bar item. |
| Update failure | Test a read-only app directory, failed download, failed rename, and an app that exits before Cocoa finishes launching. No quit on download/verification errors; installation/startup errors must restore the previous app and report the failure. |
| Duplicate launch | Start the app repeatedly and overlap login/manual launch; exactly one process and menu-bar item remain. |

Record the macOS version, architecture, IME/version, tested apps, and result in the release notes.
