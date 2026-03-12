---
# csl26-8g5m
title: 'Expand behavior report: metadata, multilingual, sort_oracle'
status: completed
type: task
priority: normal
created_at: 2026-03-12T19:01:29Z
updated_at: 2026-03-12T19:06:26Z
---

Add announce_behavior calls to metadata.rs, multilingual.rs, and sort_oracle.rs engine integration suites. Register all three in PILOT_SOURCES and the default test-report.sh target list. Regenerate the report to confirm useful human-readable output.\n\nRef skill: .claude/skills/engine-behavior-reporting/SKILL.md

## Summary of Changes

- Added `announce_behavior` calls to `metadata.rs` (12 tests), `multilingual.rs` (5 tests), and `sort_oracle.rs` (6 tests).
- Added `mod common; use common::announce_behavior;` to multilingual and sort_oracle suites.
- Registered all three in `PILOT_SOURCES` in `generate-test-report.py`.
- Added `--test metadata --test multilingual --test sort_oracle` to default `test-report.sh` target list.
- Report now covers 104 scenarios total across 7 suites.
