---
# csl26-iqxu
title: Set entry-suffix for GB/T 7714 styles (missing terminal period)
status: completed
type: bug
priority: high
created_at: 2026-07-23T20:54:31Z
updated_at: 2026-07-23T22:51:12Z
---

gb-t-7714-2025-base.yaml has no bibliography.options.entry-suffix set, so ~87% of bibliography entries on real-world corpora (no dimensions/url/cstr/doi field) are missing the terminal period the GB/T 7714-2025 standard's own worked examples (data/GB-T_7714-2025.original.toml) and real Zotero output both require. Fix: bibliography.options.entry-suffix: '.' with entry-suffix-after-url: true and entry-suffix-after-doi: true on gb-t-7714-2025-base.yaml (inherited by numeric/author-date/note). The mechanism already exists (crates/citum-schema-style/src/options/bibliography.rs:34-57) and is type-independent -- just unset for GB/T. Verify TerminalLink guard (crates/citum-engine/src/render/bibliography.rs:182-189) doesn't double-punctuate the ~43/344 entries that already end correctly. Needs a native fixture covering a 'bare' entry (no url/cstr/doi). This fix should land, and a release be cut, before asking gb7714-bench PR #25 to bump its CITUM_VERSION pin -- see docs/architecture/audits/2026-07-23_GB7714_BENCH_COMPARISON.md for full quantification (~7x exact-match rate vs Zotero on the gb7714-bench benchmark).

## Summary of Changes

Set `bibliography.options.entry-suffix: '.'`, `entry-suffix-after-url: true`,
`entry-suffix-after-doi: true` on `gb-t-7714-2025-base.yaml` (inherited by
numeric/author-date/note). Verified: Hawking fixture (ITEM-2, bare book, no
url/cstr/doi) now renders with trailing period; URL-ending pinned-corpus entry
(gbt7714.7.5.2.3:3) confirmed not double-punctuated (TerminalLink guard works
as designed). Updated 11 pre-existing test assertions across
date_annotations.rs and multilingual.rs that hardcoded the old no-period (and
in a few cases trailing-space) output. Added 2 new dedicated regression tests
to multilingual.rs: bare-entry-gets-period and url-entry-not-double-punctuated
(natively-constructed InputReference, no CSL-JSON round-trip).

Full pre-commit gate green: cargo fmt --check, cargo clippy --all-targets
--all-features -- -D warnings (no issues), cargo nextest run (2163/2163 pass).

Note: node scripts/report-core.js --style gb-t-7714-2025-numeric still
reports the pre-fix numbers (fidelityScore 0.989, 193/203) after this change --
confirmed this is the CSL-M oracle fixture staleness tracked separately in
csl26-7jib (our local oracle copy shares the missing-period defect, so it
can't see this fix land), not a problem with this fix. Verified directly via
the CLI instead.

Landed as PR #1089 (merged 2026-07-23T21:44:31Z). This bean itself was created
on the docs-audit branch (PR #1088) before the fix branch existed, so it
couldn't be marked completed until now -- PR #1089 merged before #1088, so
this bean briefly landed on main showing todo despite the fix already being
in. Status corrected here, no code change.
