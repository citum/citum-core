---
# csl26-bfa9
title: 'Fix release pipeline: drop macos-13 runner for darwin x86_64 build'
status: in-progress
type: bug
priority: high
created_at: 2026-05-18T17:07:09Z
updated_at: 2026-05-18T17:07:53Z
---

## Problem

Release workflow (.github/workflows/release.yml) pins x86_64-apple-darwin build to os: macos-13. As of v0.53.0 (run 26045039933), this job hangs indefinitely on runner queue — 55 minutes before cancellation while every other matrix leg finished in 7-14 min. GitHub has wound down macos-13 hosted runner capacity.

Net effect: v0.53.0 tag exists, but no GitHub Release, no crates.io publish, no JSR publish ever happened.

## Tasks

- [x] Branch chore/darwin-x86-runner from main
- [x] Edit .github/workflows/release.yml: swap macos-13 → macos-latest for x86_64-apple-darwin
- [ ] Commit, push, open PR with copilot review
- [ ] After merge: delete remote v0.53.0 tag (CONFIRM), retag at fixed commit, push
- [ ] Watch release workflow run, verify all 5 build jobs + release + publish-crates + publish-jsr succeed
- [ ] Confirm GitHub Release exists, crates.io shows 0.53.0, JSR shows 0.53.0

## Rationale

Apple Silicon runners cross-build x86_64-apple-darwin natively (universal Xcode linker). Workspace has no build.rs, no native C deps, only rustls TLS. scripts/release-binary.sh already calls rustup target add on non-cross legs, so no script change needed.
