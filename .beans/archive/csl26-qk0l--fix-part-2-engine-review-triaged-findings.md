---
# csl26-qk0l
title: Fix part-2 engine review triaged findings
status: completed
type: task
priority: normal
created_at: 2026-07-04T15:49:57Z
updated_at: 2026-07-04T17:24:51Z
---

Implement approved triage of audit csl26-inb7 (PR #1002, branch audit/citum-engine-review-part2-2026-07). Fix F1, F20, F2, F3, F4, F8, F17 as follow-up commits; beans for the rest; disposition table appended to the part-2 audit doc.

- [x] Commit 1: F1 markdown body-relative offsets (a32d3a5a)
- [x] Commit 2: F20 line-anchored frontmatter delimiters (4029bb1a)
- [x] Commit 3: F2 anonymous-entry policy option (57143fba)
- [x] Commit 4: F3 HTML text/data-ref escaping (f28f931c)
- [x] Commit 5: F4 preserve mixed-case in case transforms (88dbbc0f)
- [x] Commit 6: F8 et-al delimiter joins (19f8abf0)
- [x] Commit 7: F17 LaTeX href escaping (080648f8)
- [x] Commit 8: follow-up beans: csl26-k6ty, -3m45, -ebs3, -mc0c, -c361, -2ubj, -o33x, -ztxq, -92mg, -esq8, -boql, -ol1j, -ejaf; widened csl26-dr0r
- [x] Commit 9: disposition table in audit doc
- [x] Push + CI green on PR #1002 (all 14 checks pass)

## Summary of Changes

All triaged part-2 findings addressed on PR #1002: seven fix commits
(F1 markdown offset panic, F20 frontmatter anchoring, F2 anonymous-entry
option gating with zero fidelity movement across 152 styles, F3 HTML
escaping, F4 CSL case semantics, F8 et-al delimiters verified against
citeproc-js, F17 LaTeX href escaping), fourteen beans for deferred
findings (incl. late-caught F18), csl26-dr0r widened, and a disposition
table in the audit doc mapping all 22 findings to a commit or bean.
CI fully green. Merge is the user's action.
