---
# csl26-js4h
title: Component-selectable install.sh
status: completed
type: feature
priority: normal
created_at: 2026-05-22T11:48:25Z
updated_at: 2026-05-22T11:54:43Z
---

Allow users to choose which Citum binaries to install via CITUM_COMPONENTS env var.

## Plan

See PR #776 description.

## Todo

- [x] Add citum-migrate to release-binary.sh build + tarball
- [x] Add CITUM_COMPONENTS selection to install.sh (default: citum only)
- [x] Update install docs (docs/developer.html)
- [x] Selection logic matrix test (default/single/pair/all/whitespace/bogus/empty)
- [x] Installer matrix verified (same as above)
- [x] sh -n / bash -n parse-checked (shellcheck not installed locally)
- [ ] Open PR

## Notes

Default = citum only; behavior change for existing curl|sh users (lose auto citum-server). Flag in PR description.

## Summary of Changes

- `scripts/release-binary.sh`: build & ship `citum-migrate` alongside `citum` + `citum-server` in every release tarball.
- `scripts/install.sh`: parse `CITUM_COMPONENTS` (default: `citum`), validate before download, loop-install the selected subset. `all` is the only alias.
- `docs/developer.html`: document the env var next to the curl|sh one-liner.
- PR #776 opened.

Behaviour change: existing curl|sh users no longer get `citum-server` automatically — must pass `CITUM_COMPONENTS=all` or list it explicitly. Flagged in the PR description for release notes.
