---
# csl26-iqxu
title: Set entry-suffix for GB/T 7714 styles (missing terminal period)
status: todo
type: bug
priority: high
created_at: 2026-07-23T20:54:31Z
updated_at: 2026-07-23T20:54:31Z
---

gb-t-7714-2025-base.yaml has no bibliography.options.entry-suffix set, so ~87% of bibliography entries on real-world corpora (no dimensions/url/cstr/doi field) are missing the terminal period the GB/T 7714-2025 standard's own worked examples (data/GB-T_7714-2025.original.toml) and real Zotero output both require. Fix: bibliography.options.entry-suffix: '.' with entry-suffix-after-url: true and entry-suffix-after-doi: true on gb-t-7714-2025-base.yaml (inherited by numeric/author-date/note). The mechanism already exists (crates/citum-schema-style/src/options/bibliography.rs:34-57) and is type-independent -- just unset for GB/T. Verify TerminalLink guard (crates/citum-engine/src/render/bibliography.rs:182-189) doesn't double-punctuate the ~43/344 entries that already end correctly. Needs a native fixture covering a 'bare' entry (no url/cstr/doi). This fix should land, and a release be cut, before asking gb7714-bench PR #25 to bump its CITUM_VERSION pin -- see docs/architecture/audits/2026-07-23_GB7714_BENCH_COMPARISON.md for full quantification (~7x exact-match rate vs Zotero on the gb7714-bench benchmark).
