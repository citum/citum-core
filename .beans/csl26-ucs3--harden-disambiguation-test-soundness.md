---
# csl26-ucs3
title: Harden disambiguation test soundness
status: in-progress
type: task
priority: high
created_at: 2026-05-31T14:11:38Z
updated_at: 2026-05-31T14:37:29Z
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
- [ ] Open PR
- [ ] Scaffold global skill (skill-creator)
