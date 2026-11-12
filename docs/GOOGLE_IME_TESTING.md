# Google Japanese Input / Mozc verification

## Scope and results

Verification date: 2026-11-12. Host: Debian 13, x86_64. Engine: the unmodified Debian `mozc-server` package `2.29.5160.102+dfsg-1.4`, extracted into a temporary directory and run with an isolated profile.

This is a live test of the open-source Mozc engine, plus a review of its public macOS frontend. It is not a test of the installed proprietary Google Japanese Input binary on macOS. The new Carbon source detection and CoreGraphics event delivery still need a Mac build and live application checks.

| Check | Result |
| --- | --- |
| 2,196 synthetic compositions and commits | Confirmed 21 mismatches with the app's generic length estimate. Google input now bypasses that estimate entirely. All recorded engine outputs replay successfully. |
| Native raw-text commit for the same 2,196 inputs | All return the exact original ASCII input; a repeated commit adds no duplicate text. |
| 3 Shift settings × 3 input modes × 7 strings | All 63 native commits return the original input, including uppercase, half-width voiced kana, and mixed Latin/kana. |
| Rust regression suite | 34 tests pass, including 4,968 sequential conversion attempts and Google dispatch at timing boundaries. |
| Apple Silicon Rust integration | `rustc --target aarch64-apple-darwin --emit=metadata -D warnings` passes. This type-checks the Rust event bridge, without compiling or linking Objective-C. |
| macOS Google Japanese Input, TextEdit/Terminal/browser/Electron | Not executed on this Linux host. Required before release. |

The committed `tests/fixtures/mozc-composition.tsv` contains only synthetic inputs and the strings returned by the engine. It includes alphabetic table entries, explicit edge cases, and deterministic generated sequences (seed 41). It is independent of the app's conversion table.

## Bugs found and changes

`nn` commits one `ん` immediately in Mozc. Consequently `nnya` produces `んや` (2 characters), not `んにゃ` (3). The old estimate could delete one character before the intended range. `konnya` similarly produces `こんや`; `konnnya` is needed for `こんにゃ`.

An invalid prefix is flushed as a chunk by Mozc. For example, `xti` produces `ｘｔい` and `lto` produces `ｌｔお`. The app's generic parser instead retries part of the failed prefix as a new syllable, undercounting both examples. Google input now delegates this to the IME; Google-specific parsing rules are not imposed on the non-Google fallback, whose actual behavior still needs testing with the relevant IME.

Shift behavior is configurable. For example, `Kya` can remain ASCII rather than becoming two kana. Lowercasing the physical keys cannot safely estimate the displayed length, so non-native replacement skips Shift-dependent input.

Most importantly, the public macOS controller does not always commit a Japanese composition on the first Eisu key. With marked text it switches to `HALF_ASCII`; a native Eisu double tap sends `COMMIT_RAW_TEXT`. Thus the app's previous assumption that it could safely delete a committed Japanese word after the first tap was invalid for Google Japanese Input.

Google/Mozc inputs now use the native Eisu path rather than Accessibility replacement or synthetic Delete/retyping. The provider observed at the first left tap is retained for the pending double tap, even if the input source changes. On the second tap, the code posts a fresh pair of Eisu events, intended to stay within Google's 500 ms native window while preserving cmd-kana's inclusive 550 ms window. The dispatch decision and engine commands are tested; actual source detection, event timing, and frontend behavior require macOS integration validation.

Public implementation references, pinned to the tested engine version:

- [macOS Eisu/Kana handling and the 500 ms interval](https://github.com/google/mozc/blob/2.29.5160.102/src/mac/GoogleJapaneseInputController.mm)
- [macOS Shift/key translation](https://github.com/google/mozc/blob/2.29.5160.102/src/mac/KeyCodeMap.mm)
- [Engine commands and modes](https://github.com/google/mozc/blob/2.29.5160.102/src/protocol/commands.proto)
- [Configurable Shift behavior](https://github.com/google/mozc/blob/2.29.5160.102/src/protocol/config.proto)

## Remaining macOS checks

Build with `./scripts/check.sh`, then record the macOS version, hardware, Google Japanese Input version, and application for each result. Use synthetic text with a visible unrelated prefix (for example `PREFIX `) to detect unintended deletion.

1. In Google Hiragana mode, type `aka`, `konnnitiha`, `nnya`, `xti`, and `kfa`. Double-tap left Command at ordinary speed and near 499, 500, and 550 ms. Verify the exact original ASCII appears once and the prefix survives.
2. Repeat with each Shift setting and with full-width/half-width Katakana modes. Include `Kya` and voiced kana.
3. Repeat after Delete, with more than 128 keys, with an empty composition, and after Enter/Space confirms or converts a word. Compare to manually pressing native Eisu twice; already committed text must not be reconstructed from stale cmd-kana history.
4. Verify the input source after the first tap, then test rapid successive conversions, left single taps, right single/double taps, and Command shortcuts. No reconversion should appear after right double taps.
5. Run the same checks in TextEdit, Terminal, Safari, Chrome, and an Electron editor, including controls that do not expose Accessibility text replacement. Verify actual marked-text/commit behavior and permissions.

Passing the Linux engine checks alone does not establish that these macOS checks pass or that every Google Japanese Input version/custom romaji table is supported.
