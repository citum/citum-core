---
# csl26-0a2l
title: 'Doc: clarify BDD vs unit test style rule'
status: completed
type: task
priority: normal
created_at: 2026-03-16T19:44:13Z
updated_at: 2026-03-16T19:55:27Z
---

Add a Test Style section to CODING_STANDARDS.md and a pointer in CLAUDE.md so builders know when to use BDD naming (given/when/then + rstest) vs plain #[test].

## Summary of Changes

- Added Test Style section to docs/guides/CODING_STANDARDS.md
- Added one-line pointer in CLAUDE.md Verification section
- Added Test Style table to .claude/skills/test-coverage/SKILL.md
- Converted test_resolve_multilingual_name_transliteration_priority to #[rstest] with two named cases
- Renamed single-scenario tests to concise descriptive names (no BDD prefix)
