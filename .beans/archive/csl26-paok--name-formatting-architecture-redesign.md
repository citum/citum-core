---
# csl26-paok
title: Name formatting architecture redesign
status: completed
type: feature
priority: high
created_at: 2026-03-05T13:48:49Z
updated_at: 2026-03-09T20:12:33Z
---

Remove ModeDependent from AndOptions; add NameForm enum (Full|FamilyOnly|Initials) to ContributorConfig; migrate all affected styles; write architecture doc. Breaking change — implement on a PR branch.

## Tasks
- [x] Add NameForm enum to citum-schema contributors.rs
- [x] Add name_form: Option<NameForm> to ContributorConfig + update merge()
- [x] Remove AndOptions::ModeDependent variant
- [x] Remove ModeDependent resolver loop in contributor.rs
- [x] Implement NameForm dispatch in given-name rendering path
- [x] Migrate apa-7th.yaml: replace mode-dependent with integral.options.contributors.and
- [x] Migrate chicago-notes.yaml: add name_form=family-only to subsequent block
- [x] Audit all other styles for name_form needs
- [x] Write docs/architecture/NAME_FORMATTING.md
- [ ] Oracle + batch-aggregate verification; update baseline if scores improve
