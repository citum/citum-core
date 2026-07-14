---
# csl26-l5u9
title: citum-migrate missing from Linux release tarballs (musl has no V8 prebuilt)
status: completed
type: bug
priority: high
tags:
    - infra
    - migrate
    - cli
created_at: 2026-07-14T13:38:37Z
updated_at: 2026-07-14T13:53:08Z
---

Issue #1054: citum-migrate is silently skipped on *-linux-musl release targets because rusty_v8 has no musl prebuilt. Fix: add x86_64/aarch64-unknown-linux-gnu targets to the release matrix (glibc has V8 prebuilts) and have install.sh fetch citum-migrate from the gnu tarball on Linux instead of skipping it.

## Summary of Changes

Root cause: `scripts/release-binary.sh` skips building `citum-migrate` for
`*-linux-musl` release targets because `deno_core`/`rusty_v8` has no
prebuilt musl static libs (only glibc, macOS, Windows). `install.sh` already
warned about this and suggested `cargo install`, but users who explicitly
requested `citum-migrate` via `CITUM_COMPONENTS` still silently got nothing
installable from the prebuilt path.

Fix:
- Added `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` to the
  release build matrix (`.github/workflows/release.yml`) — glibc targets
  have V8 prebuilts, so `citum-migrate` builds there.
- `scripts/install.sh` now fetches `citum-migrate` from the gnu tarball as a
  fallback when installing on a musl target and `citum-migrate` was
  requested (`migrate_fallback_target`, `fetch_tarball`), instead of
  skipping it. Falls back to the original warn + `cargo install` message if
  the gnu tarball itself can't be fetched (offline mirror, pre-fix release).
- Updated `scripts/release-binary.sh`'s comment to document the gnu
  fallback path (no behavioral change needed there — the existing
  `*-linux-musl` case already does the right thing for the new targets).
- Added two tests to `scripts/test_release_workflow.py` pinning the new
  matrix targets and the install.sh fallback logic.

Verified offline (network access is sandboxed in this environment) by
stubbing `curl` with a local-file-copy shell function and exercising three
paths against synthetic release tarballs: (1) musl host + gnu tarball
present → citum-migrate installed from gnu fallback; (2) musl host + gnu
tarball missing → graceful warn-and-skip, exit 0; (3) citum-migrate not
requested → no gnu fetch attempted. Caught and fixed a real bug in the
process: `fetch_tarball`'s progress messages were leaking into its
command-substitution return value via stdout — moved them to stderr.

`just pre-commit` (fmt, clippy -D warnings, nextest) passed clean: 1927/1927
tests.
