---
# csl26-5nl2
title: 'Fix GB/T c1988 misinterpretation: add native copyright/printing dates'
status: completed
type: bug
priority: high
tags:
    - style
    - fidelity
    - multilingual
    - dates
    - migrate
created_at: 2026-07-18T10:41:08Z
updated_at: 2026-07-18T11:52:56Z
parent: csl26-8uxa
---

PR #1064 modeled GB/T 7714 c1988 as EDTF circa/approximate, but the c means copyright, not circa (reviewer comment on PR #1064). Revert the circa hack, add native copyright and printing date variables, fix estimated-date bracket rendering, revert range-bracket wrapping, and revisit the div-009 tex.cstr divergence.

## Checklist

- [x] Commit 1: revert circa misinterpretation (`normalize_csl_circa_literal`, contract test, GB/T approximation-marker)
- [x] Commit 2: revert range bracket wrapping (`format_closed_range`, `range_end_prefix/suffix`, GB/T YAMLs); keep note-field range parsing
- [x] Commit 3: native `copyright` date variable (enum, resolver, accessor, migration, GB/T fallback chain)
- [x] Commit 4: estimated-date brackets (`approximation_marker_suffix`) + native `printing` date variable
- [x] Commit 5: revisit `div-009` tex.cstr divergence (documented reviewer's §7.9.1/§7.9.2 alternate reading in verification-policy.yaml; omission fix split to follow-up bean csl26-ia43, blocked on reviewer confirmation)
- [x] Update `docs/specs/DATE_MODEL.md` with copyright/printing date variables and the §7.5.4.3 four-case mapping
- [x] `just schema-gen` if citum-schema* changed
- [x] `just pre-commit` green
- [x] GB/T oracle confirms all four fixtures render correctly (copyright/printing/estimated/range), with documented pending-reviewer divergence for 3 of 4 vs the pinned oracle
- [x] Push branch, `gh pr checks --watch` green (do not merge)

## Oracle divergence note (pending reviewer sign-off)

The pinned CSL-M oracle (`tests/fixtures/csl-m/gb-t-7714-2025-numeric.csl` +
`tests/fixtures/test-items-library/gb-t-7714-2025.json`) does NOT match the
GB/T 7714 §7.5.4.3 standard-conformant rendering the reviewer described for
three of the four cases:

- Copyright (`c1988`): oracle wants `c1988` — matches; Commit 3 fixes this correctly.
- Printing (`1995印刷`): oracle currently matches plain `1995` (no 印刷 suffix).
- Estimated (`1936~` → `[1936]`): oracle currently matches plain `1936` (no brackets).
- Range (`1957/1990`): oracle wants the bracketed `1957—[1990]`, not the
  reviewer-suspected first-run artifact `1957—1990`.

Per Bruce (2026-07-18): defer to the reviewer over the pinned oracle here —
CSL has no native concept for these GB/T substitute-year forms, so some
hackery is expected either way. Proceeding to implement printing suffix +
estimated brackets per the reviewer's described standard behavior, and kept
the range-bracket revert (Commit 2) as-is. This means the branch will show a
raw fidelity regression against `gb-t-7714-2025-numeric.csl` for rows
2/3/4 vs `main` — expected and to be called out explicitly in the PR
description for reviewer confirmation before merge.

## Summary of Changes

Reverted the c1988 circa misread from PR #1064 (commit d56ac1a1) and its
paired range-bracket wrapping. Added native `copyright` and `printing`
date variables end to end (schema enum, engine resolution, Monograph
fields, migration routing) and estimated-date bracket rendering via a new
`approximation-marker-suffix` style option. All 6 commits pass
`just pre-commit` (2054 tests). Rebuilt commit history once after the
pre-push hook rejected `gb-t`/`specs` as invalid conventional-commit
scopes (not in the allowed list) — corrected to `schema`/`spec`/`migrate`.

Known, deliberate divergence from the pinned `gb-t-7714-2025-numeric.csl`
oracle for 3 of 4 target cases (printing suffix, estimated brackets, range
un-bracketing) — documented in the PR description and this bean for
reviewer confirmation before merge, per Bruce's explicit direction to
defer to the reviewer over the pinned oracle here.

Deferred: URL-containment-based CSTR identifier omission (div-009,
§7.9.1/§7.9.2) — split to draft bean csl26-ia43, blocked on reviewer
confirmation of the interpretation.
