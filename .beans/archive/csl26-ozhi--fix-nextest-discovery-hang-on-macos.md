---
# csl26-ozhi
title: Fix nextest discovery hang on macOS
status: completed
type: bug
priority: normal
created_at: 2026-03-20T01:43:01Z
updated_at: 2026-03-20T01:43:11Z
---

## Summary of Changes

- Added `.cargo/config.toml` with `[target.aarch64-apple-darwin] linker = "scripts/macos-link.sh"`
- Added `scripts/macos-link.sh`: wraps `cc`, re-signs Mach-O output with CS_ADHOC (flags 0x2, removes CS_LINKER_SIGNED / 0x20000 bit), and pre-warms the macOS security cache by invoking the binary with `--list` in background.

Root cause: macOS 15+ runs a ~60s XProtect/amfid scan on first execution of CS_LINKER_SIGNED binaries. Re-signing with CS_ADHOC reduces per-binary scan time to ~31s (4× fewer 16 KB hash pages vs 4 KB). Background pre-warm overlaps with the build so scan completes before nextest runs.

Refs: GitHub #406
