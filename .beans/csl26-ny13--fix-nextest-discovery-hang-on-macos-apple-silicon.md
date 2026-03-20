---
# csl26-ny13
title: Fix nextest discovery hang on macOS Apple Silicon
status: in-progress
type: bug
created_at: 2026-03-20T01:42:50Z
updated_at: 2026-03-20T01:42:50Z
---

macOS security daemon (amfid/syspolicyd) runs a ~60s XProtect scan on first execution of freshly-linked binaries carrying CS_LINKER_SIGNED ad-hoc signatures (flags 0x20002). This makes cargo nextest list appear to hang for several minutes after a fresh build on macOS 15+ / Apple Silicon.

Fix: add .cargo/config.toml with a custom linker wrapper for aarch64-apple-darwin. The wrapper re-signs Mach-O executables with a proper CS_ADHOC signature (flags 0x2, no CS_LINKER_SIGNED bit) and immediately invokes the binary with --list in the background to trigger the macOS scan during the build rather than at nextest list time.

Result: first-run latency drops from ~60s to ~31s per binary (4× fewer hash pages to verify), and the background pre-warm overlaps with the remainder of the build.

See: GitHub issue #406
