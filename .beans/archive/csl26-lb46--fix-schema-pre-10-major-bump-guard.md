---
# csl26-lb46
title: 'fix: schema pre-1.0 major bump guard'
status: done
type: bug
created_at: 2026-03-25T06:17:03Z
updated_at: 2026-03-25T06:17:03Z
---

bump_version() in prepare-release-artifacts.py has no pre-1.0 guard. Schema-Bump: major on feat(template-v2) caused 0.12.0 → 1.0.0 instead of 0.13.0. Fix: treat major as minor when current major==0. Also revert STYLE_SCHEMA_VERSION and docs/schemas/style.json to 0.13.0, update SCHEMA_VERSIONING.md, and regenerate schema JSONs.
