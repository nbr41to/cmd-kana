# Contributing

## Changes

Keep the event-processing path small and isolate testable decisions in pure Rust. Avoid adding platform fallbacks unless the primary path has a documented compatibility gap and both paths have a safe failure mode.

Before submitting a change:

```sh
./scripts/check.sh
```

Add a regression test for every input-state or romaji-conversion bug. UI-only changes must not modify `src/input.rs` unless the input behavior intentionally changes.

## Pull requests

Describe the macOS version, hardware architecture, keyboard layout, IME, affected applications, reproduction steps, and manual test results. Do not include captured private input or unrelated logs.
