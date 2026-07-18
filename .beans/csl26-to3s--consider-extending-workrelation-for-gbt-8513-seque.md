---
# csl26-to3s
title: Consider extending WorkRelation for GB/T §8.5.1.3 sequel citations
status: draft
type: task
priority: low
tags:
    - style
    - migrate
created_at: 2026-07-18T14:16:03Z
updated_at: 2026-07-18T14:16:15Z
parent: csl26-8uxa
---

GB/T 7714—2025 §8.5.1.3 sequel citations (serially-published parts appended after the first) are currently handled by a documented convention only (cram into SerialComponent.pages, see docs/reference/GBT_7714_CITATION_CONVENTIONS.md) per reviewer @YDX-2147483647's explicit recommendation not to overfit this rare case on PR #1067. This bean captures the extension options if demand for structured rendering ever appears.

## Options

**Option 1 (current, zero code):** document the convention using existing
`SerialComponent.pages`/`note` fields, matching what gbt7714-bibtex-style
and biblatex-gb7714-2025 already do. See
`docs/reference/GBT_7714_CITATION_CONVENTIONS.md`. Opaque text — no
per-part sorting/localization/delimiter styling.

**Option 2 (middle ground, ~5-7 files):** add `sequels: Vec<WorkRelation>`
on `SerialComponent`, reusing the existing `original`/embedded-relation
pattern (an accessor that walks embedded parts pulling
`issued`/`volume`/`issue`/`pages`, a value resolver, a template variable
joining with `; `). Matches the `original`/`container` idiom Citum already
uses for related-work fields
(`crates/citum-schema-data/src/reference/accessors.rs` `original_embedded`,
`original_date`, `original_title` etc.). No new relation variant or public
type.

**Option 3 (most complete, ~8-12 files):** a dedicated
`SequelPart { year, volume, issue, pages }` struct in
`crates/citum-schema-data/src/reference/types/common.rs` alongside
`Numbering`, with its own accessor, value resolver, template variable,
JSON-schema/specta derives, and conversion support. Structured/sortable/
localizable, but the most net-new schema surface for a case the reviewer
called "quite rare."

**Recommendation if ever revisited:** Option 2 is the closest fit to
Citum's existing idiom, but Option 1 already reaches parity with the
reference LaTeX implementations at zero cost — only pursue Option 2/3 if
concrete demand (a real corpus item needing structured rendering)
materializes.
