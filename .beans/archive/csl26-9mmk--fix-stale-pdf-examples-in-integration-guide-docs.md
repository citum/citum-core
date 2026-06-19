---
# csl26-9mmk
title: Fix stale --pdf examples in integration guide docs
status: completed
type: bug
priority: normal
created_at: 2026-06-19T12:17:39Z
updated_at: 2026-06-19T12:18:21Z
---

djot.html and scholar-cli.html both show a broken --pdf example. The flag requires -f typst (CLI errors otherwise) and is not available in cargo-install builds. Fix both pages to use the two-step typst compile workflow.

## Summary of Changes

Replaced broken `--pdf` examples in two integration guide pages with the correct two-step Typst workflow (`-f typst` + `typst compile`). The `--pdf` flag is not available in `cargo install` builds and requires `-f typst` even when it is available.
