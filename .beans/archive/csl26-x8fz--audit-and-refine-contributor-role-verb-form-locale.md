---
# csl26-x8fz
title: Audit and refine contributor role verb-form locale terms
status: completed
type: task
priority: normal
created_at: 2026-03-28T10:17:49Z
updated_at: 2026-03-28T11:10:34Z
---

CSL locale has 'interview by' for interviewer verb form, but Citum locale has 'interviewed by'. Need to audit all contributor role verb-form terms in locale against CSL locales-en-US.xml to ensure parity. This affects styles using form: verb for contributor roles.

## Summary of Changes

- en-US: added missing `verb: with guest` to guest role
- fr-FR: fixed silently-broken SingularPlural verb object → plain string
- en-US: added inline comment to interviewer.verb (upstream CSL typo)
- DIVERGENCE_REGISTER: added div-007 for interviewer verb-form departure
- PR: citum/citum-core#464
