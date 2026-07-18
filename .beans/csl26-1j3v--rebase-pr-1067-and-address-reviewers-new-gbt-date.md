---
# csl26-1j3v
title: 'Rebase PR #1067 and address reviewer''s new GB/T date feedback'
status: in-progress
type: task
priority: high
tags:
    - style
    - fidelity
    - multilingual
    - dates
    - migrate
created_at: 2026-07-18T13:20:11Z
updated_at: 2026-07-18T14:16:43Z
parent: csl26-8uxa
---

PR #1067 needs rebase onto main (unrelated CJK-punctuation work landed). Reviewer confirmed the divergence table and raised two new GB/T 7714 date cases: ancient/regnal years (§7.5.4.1) and sequel/serial-continuation citations (§8.5.1.3). Rebase, document the sequel convention, and file follow-up beans for both new cases.

## Checklist

- [x] Rebase `fix/gb-t-copyright-date` onto `origin/main`, resolve the single `verification-policy.yaml` conflict (div-009/div-010 adjacency)
- [x] Regenerate `docs/schemas/style.json` via `just schema-gen` (no diff vs rebase auto-merge)
- [x] `just pre-commit` green post-rebase (2058 tests)
- [x] Oracle spot-check unchanged for the four target fixtures (copyright matches; printing/estimated/range diverge as intended, no new regressions vs main baseline of 146/203)
- [x] Confirm with user, then force-push-with-lease
- [ ] `gh pr checks 1067 --watch` green again
- [x] Document the §8.5.1.3 sequel convention (zero code) — docs/reference/GBT_7714_CITATION_CONVENTIONS.md
- [x] File ancient/regnal-years bean (§7.5.4.1) — csl26-0kqf
- [x] File sequel-extension-options bean (§8.5.1.3) — csl26-to3s
- [ ] Confirm reply wording with user, post PR comment (do not merge)
