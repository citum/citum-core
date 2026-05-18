---
# csl26-bfa9
title: 'Fix release pipeline: drop macos-13 runner for darwin x86_64 build'
status: completed
type: bug
priority: high
created_at: 2026-05-18T17:07:09Z
updated_at: 2026-05-18T17:52:43Z
---

## Problem

Release workflow (.github/workflows/release.yml) pins x86_64-apple-darwin build to os: macos-13. As of v0.53.0 (run 26045039933), this job hangs indefinitely on runner queue — 55 minutes before cancellation while every other matrix leg finished in 7-14 min. GitHub has wound down macos-13 hosted runner capacity.

Net effect: v0.53.0 tag exists, but no GitHub Release, no crates.io publish, no JSR publish ever happened.

## Tasks

- [x] Branch chore/darwin-x86-runner from main
- [x] Edit .github/workflows/release.yml: swap macos-13 → macos-latest for x86_64-apple-darwin
- [x] Commit, push, open PR with copilot review (#744)
- [x] After merge: delete remote v0.53.0 tag (CONFIRM), retag at fixed commit d27cb300, push
- [ ] Watch release workflow run, verify all 5 build jobs + release + publish-crates + publish-jsr succeed
- [ ] Confirm GitHub Release exists, crates.io shows 0.53.0, JSR shows 0.53.0

## Rationale

Apple Silicon runners cross-build x86_64-apple-darwin natively (universal Xcode linker). Workspace has no build.rs, no native C deps, only rustls TLS. scripts/release-binary.sh already calls rustup target add on non-cross legs, so no script change needed.



## Summary of Changes

`.github/workflows/release.yml` matrix swap: `x86_64-apple-darwin` from `os: macos-13` → `os: macos-latest`. Apple Silicon cross-builds x86_64 via the universal Xcode linker; `scripts/release-binary.sh` already runs `rustup target add` for non-cross legs.

Verification: v0.53.0 retag triggered the fixed pipeline; the x86_64-apple-darwin job completed in ~14 min (vs the original 55-min hang). All five build matrix legs succeeded.

Downstream publish failures (publish-crates / publish-jsr) surfaced unrelated release-pipeline gaps tracked separately in [[csl26-93wp]].
