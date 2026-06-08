---
# csl26-effd
title: 'Reconcile bibliography grouping: vocabulary, spec, engine de-dup'
status: completed
type: task
priority: high
created_at: 2026-06-08T17:38:34Z
updated_at: 2026-06-08T17:58:12Z
---

Follow-up to fd0c6eee. Three-part cleanup:

Part A — Engine de-dup: fold `process_document_with_frontmatter_groups` onto the `render_document_bibliography_blocks` primitive (same path the CLI/server already use); delete dead `render_document_bibliography_groups` / `render_with_custom_groups`; collapse the three process_document_with_* entry points to two real strategies (trailing ordered sections, positioned fenced-divs).

Part B — Spec reconciliation: promote BIBLIOGRAPHY_GROUPING.md to Active as the single mechanism owner (real types/primitive, vocabulary rule, input-surfaces table, grouping-vs-partitioning subsection); slim PER_DOCUMENT_CONFIG_OVERRIDES.md to add the missing CLI/server per-document surfaces and reference BIBLIOGRAPHY_GROUPING for the mechanism; cross-link in SERVER_INTERACTIVE_API.md.

Vocabulary decision: group = definition (BibliographyGroup), block = placed instance (BibliographyBlockRequest). No schema-type or serde rename.

Related: csl26-group, csl26-o8ji, fd0c6eee

## Summary of Changes

Part A (engine, commit 994707ce): Folded  onto  primitive. Deleted dead  and . Deleted  (replaced by  which emits format-correct headings at H2 level). Updated 3 tests: group headings promoted from H1 to H2, unassigned-fallback removed (explicit over magic).

Part B (docs, this commit): Promoted BIBLIOGRAPHY_GROUPING.md from Design → Active. Rewrote spec with real types (, ), vocabulary rule (group = definition, block = placed instance), input-surfaces table (4 surfaces → 1 renderer, precedence order), grouping-vs-partitioning distinction, unmatched-entry policy. Slimmed PER_DOCUMENT_CONFIG_OVERRIDES.md: replaced re-described BibliographyGroup prose with reference to BIBLIOGRAPHY_GROUPING.md; added frontmatter  list, CLI , and server  surfaces. Added cross-links in SERVER_INTERACTIVE_API.md.
