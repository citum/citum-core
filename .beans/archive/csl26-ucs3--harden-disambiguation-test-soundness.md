---
# csl26-ucs3
title: Harden disambiguation test soundness
status: completed
type: task
priority: high
created_at: 2026-05-31T14:11:38Z
updated_at: 2026-05-31T14:41:14Z
---

Fix 1 broken + 5 suspicious disambiguation tests, add 2 coverage-gap tests (§4 multilingual key, §2 givenname rule), persist JSON analysis as audit doc, scaffold global skill. Ref docs/specs/DISAMBIGUATION.md.

- [x] Branch fix/disambiguation-test-soundness
- [x] Fix broken: subsequent_et_al_thresholds exact assertions
- [x] Rename/realign suspicious: duplicate_family_names, initials_collide, conditions_expand, et_al_subsequent_form
- [x] Strengthen: group-local suffix restart assertions (document.rs:822)
- [x] New: §4 multilingual-key collision test (original-name form; transliteration path noted as unimplemented)
- [x] New: §2 positive given-name expansion test (by-cite/all-names rule not in schema; positive collision case covered)
- [x] Audit doc docs/architecture/audits/2026-05-31_DISAMBIGUATION_TEST_SOUNDNESS.md
- [x] Pre-commit gate green
- [x] Open PR (#850)
- [x] Scaffold global skill (skill-creator) → ~/.claude/skills/test-soundness-review/

## Summary of Changes

Fixed 1 broken test (exact assert_eq! replacing substring checks), renamed/realigned 5 suspicious tests, strengthened the group-local suffix restart assertion in document.rs, added 3 new tests (positive given-name expansion, givenname cascade fallback, multilingual original-name collision). Wrote audit doc to docs/architecture/audits/2026-05-31_DISAMBIGUATION_TEST_SOUNDNESS.md. Corrected DISAMBIGUATION.md §4 acceptance criterion from [x] to [ ] — render_name_for_disambiguation is unimplemented. Scaffolded global skill at ~/.claude/skills/test-soundness-review/. PR #850 open.
