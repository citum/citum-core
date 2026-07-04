---
# csl26-vq1g
title: Fix engine review high + selected medium findings
status: completed
type: task
priority: normal
created_at: 2026-07-04T02:02:06Z
updated_at: 2026-07-04T10:38:01Z
---

Implement approved plan from audit csl26-nj72 on branch audit/citum-engine-review-2026-07. Commits: (1) process_document returns Result, (2) no-date-form style option replaces harvard hardcode, (3) headings via OutputFormat, (4) id stubs in custom-group path, (5) warning scanner sub-spec recursion, (6) non-UTF-8 refs input error, (7) crate CLAUDE.md refresh, (8) follow-up beans. Then open PR.

- [x] Commit 1: process_document Result (6f10c907)
- [x] Commit 2: no-date-form option + schema-gen (02197175)
- [x] Commit 3: OutputFormat headings (e0609b1f)
- [x] Commit 4: sorted_id_stubs (69f5a397)
- [x] Commit 5: warnings sub-spec scan (635101c8)
- [x] Commit 6: refs input UTF-8 (b7662f8c)
- [x] Commit 7: CLAUDE.md refresh (06f548a0)
- [x] Commit 8: follow-up beans (csl26-aawl, csl26-54bk, csl26-dog9, csl26-wj7z, csl26-b801, csl26-qi7l, csl26-wfua, csl26-dr0r)
- [x] Push branch + open PR + CI green (PR #1001, all checks pass)

## Summary of Changes

All planned commits landed on audit/citum-engine-review-2026-07 and PR #1001
opened with all CI checks green (semver, fidelity, hygiene, release dry runs,
rust lint+test, security audit, WASM smoke). Both high findings fixed
(process_document returns Result; no-date-form style option replaces the
harvard hardcode), four medium findings fixed (OutputFormat headings, id
stubs, sub-spec warning scan, non-UTF-8 refs error), crate CLAUDE.md
refreshed, and eight follow-up beans filed for deferred findings. Merge is
the user's action.
