# Changelog

All notable changes are documented here. This project follows Semantic Versioning.

## [Unreleased]

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
