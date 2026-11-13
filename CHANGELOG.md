# Changelog

All notable changes are documented here. This project follows Semantic Versioning.

## [Unreleased]

## [0.10.0] - 2026-11-13

- Added on-demand GitHub Releases checks and in-app download, verification, replacement, and restart.
- Preserve the previous app until the updated app finishes launching; roll back failed installations or launches.
- Renamed the existing restart action to distinguish it from downloading an update.
- Added download, installation, privacy, license, and uninstall guidance.
- Added a verified Developer ID/notarization path for preparing a draft public release.

## [0.9.3] - 2026-11-12

- Corrected replacement lengths for small kana, extended romaji syllables, alphabetic symbol shortcuts, and consecutive `nn` pairs.
- Cancel replacement after Delete or history overflow rather than guessing the remaining IME composition or truncating a syllable.
- Track both held Command keys and suppress Command-click and modifier-only chords without leaving the next tap stuck.
- Avoid recording known English-mode input as a Japanese replacement candidate, and discard state when event monitoring is interrupted.
- Added Linux input checks and repeated conversion/timing regression coverage.
- Delegate Google Japanese Input/Mozc double taps to native Eisu conversion instead of deleting IME-owned marked text; preserve the original provider across the first Eisu switch and handle the 500–550 ms timing gap.
- Skip guessed deletion lengths for Shift-dependent input on the non-Google path. Google-specific `nn` and invalid-prefix behavior is handled by the IME itself.
- Added 2,196 recorded live Mozc composition fixtures and a reproducible engine check covering raw commits and 63 Shift/input-mode combinations.
- macOS Google Japanese Input source detection and native event delivery still require the manual integration checks in `docs/GOOGLE_IME_TESTING.md`; Linux engine checks do not establish those results.

## [0.9.2] - 2026-08-27

- Prevented rapid right Command taps from sending duplicate Kana events that could reopen IME conversion.
- Clear recorded conversion history after a right Command switch, conversion confirmation, punctuation, or new typing after an English switch.
- Prevented stale input history from expanding a later left double-tap replacement to older text left of the cursor.

## [0.9.1] - 2026-08-13

- Removed Input Monitoring and Accessibility status rows from the menu-bar menu; permission details remain available in Settings.

## [0.9.0] - 2026-08-13

- Keep the settings UI running when permissions are missing and automatically retry event monitoring after permission changes.
- Prevent duplicate cmd-kana processes with a per-user process lock.
- Added in-app troubleshooting guidance for resetting Input Monitoring and Accessibility permissions.

## [0.8.0] - 2026-08-13

- Added a settings toggle to launch cmd-kana automatically at login on macOS 13 and later.
- Show the actual macOS login-item authorization state instead of storing a duplicate preference.
- Launch silently in the menu bar when macOS starts the app as a login item.
- Corrected the declared minimum system version to macOS 11.0 to match the arm64 binary.

## [0.7.2] - 2026-08-13

- Added an Update button to the settings window so the app can restart while its menu-bar item is hidden.

## [0.7.1] - 2026-08-13

- Fixed left double-tap conversion for mixed full-width Latin and Kana output such as `ｋふぁ`.
- Added automated macOS CI, release-candidate archives, checksums, and release documentation.
- Expanded input-state and romaji edge-case tests.

## [0.7.0] - 2026-08-13

- Added a native settings window with menu-bar visibility and permission status.
- Added shortcuts to the relevant macOS privacy settings.
- Preserved the tested Command-key input behavior from 0.6.0.

## [0.6.0] - 2026-08-13

- Replaced the previous word directly through the Accessibility API where supported.
- Kept a fast event-based fallback for unsupported controls.
- Added exact physical-key-to-romaji tests.

## [0.5.0] - 2026-08-13

- Switched to native virtual Eisu and Kana key events.
- Removed Carbon input-source selection and persistence logic.

## [0.1.0] - 2026-08-13

- Initial left/right Command input switching implementation.
