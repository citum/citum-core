---
# csl26-fvpo
title: 'oracle-snapshot: include locale files in staleness hash'
status: todo
type: bug
priority: low
tags:
    - testing
    - oracle
created_at: 2026-07-16T11:16:53Z
updated_at: 2026-07-16T11:17:07Z
---

Snapshots are keyed on fixture_hash + csl_hash only (scripts/oracle-snapshot.js). Adding or editing scripts/locales-*.xml silently leaves stale snapshots that no longer match live citeproc output — found when adding locales-zh-CN.xml for GB/T 7714 (snapshots claimed current, needed --force). Include a hash of the locale files the style can request, or of the whole scripts/locales-*.xml set.
