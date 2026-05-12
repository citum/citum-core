---
# csl26-f1yz
title: Fix Codex skill migration compatibility
status: completed
type: bug
priority: high
created_at: 2026-05-11T19:40:29Z
updated_at: 2026-05-11T19:42:24Z
---

## Problem

Moving public skills from `.codex/skills/` to `.skills/` risks breaking existing Codex installs and leaves stale documentation references.

## Checklist

- [x] Preserve compatibility for existing Codex skill symlinks after the `.skills/` migration
- [x] Update install and workflow docs to reference the current installer and behavior
- [x] Make public skill descriptions agent-neutral where they still say Codex
- [x] Run bean hygiene after updating the bean state

## Summary of Changes

- Added legacy `.codex/skills/*` compatibility shims that point to the canonical `.skills/` tree so existing Codex installs keep resolving after the migration.
- Updated contributor docs to use `./scripts/install-skills.sh` and clarified the compatibility story for Codex users.
- Made public skill descriptions agent-neutral where they no longer need Codex-specific branding.
