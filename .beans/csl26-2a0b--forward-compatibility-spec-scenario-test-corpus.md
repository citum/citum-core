---
# csl26-2a0b
title: Forward-compatibility spec + scenario test corpus
status: in-progress
type: task
priority: high
created_at: 2026-05-15T14:41:05Z
updated_at: 2026-05-15T14:49:05Z
---

Audit + tests + spec to confirm and document the forward-compat strategy: older engines should soft-degrade with warnings on new attribute enums / option fields / locale terms, hard-fail only on template grammar changes and unknown InputReference class.

Plan: /Users/brucedarcus/.claude/plans/can-you-confirm-ideally-wiggly-elephant.md
Related: csl26-piea, csl26-fuw7

## Tasks
- [x] Write docs/specs/FORWARD_COMPATIBILITY.md (Draft)
- [x] Add forward_compatibility integration test in crates/citum-engine/tests/
- [x] Check in initial forward_compat_gaps.snap snapshot
- [x] Cross-link from DESIGN_PRINCIPLES, SCHEMA_VERSIONING, ENUM_VOCABULARY_POLICY, EXTENSIBILITY_STRATEGY
- [x] Update bean csl26-fuw7 body to point at the new spec
- [x] File follow-up beans for each declared!=observed gap row (csl26-ld6e, csl26-0ksu, csl26-acfh, csl26-o1z5, csl26-1bdr)
- [ ] PR with snapshot summary table
