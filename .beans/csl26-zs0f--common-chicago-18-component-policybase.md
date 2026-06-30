---
# csl26-zs0f
title: Common Chicago 18 component policy/base
status: todo
type: feature
priority: high
created_at: 2026-06-30T14:29:51Z
updated_at: 2026-06-30T14:29:51Z
parent: csl26-40n4
blocked_by:
    - csl26-fr6f
---

Introduce a hidden common base for Chicago 18 covering contributor formatting, title casing/quotes/italics, page-range format, DOI/URL conventions, periodical/book/chapter/media/archive component semantics, and suppression policy (e.g. personal_communication bibliography suppression). Only where inheritance can express the commonality without forcing the wrong rendered order — author-date and notes ordering stay separate (see csl26-40n4 architecture notes).

## Todo
- [ ] Identify genuinely shared component rules from the audit (csl26-fr6f)
- [ ] Design hidden base style and extends graph
- [ ] Migrate chicago-author-date-18th and chicago-notes-18th onto the shared base
- [ ] Verify no regression via report-core.js fidelity across all 4 variants
