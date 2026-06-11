---
# csl26-cxi9
title: 'migrate: bib component order and label leakage'
status: completed
type: bug
priority: normal
created_at: 2026-06-10T16:56:45Z
updated_at: 2026-06-11T15:37:08Z
parent: csl26-vmcr
---

Cluster C3 (~8 deep-failure numeric styles) from docs/architecture/audits/2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md. Migrated bibliography entries render components in scrambled order with label/affix leakage, e.g. brazilian-journal-of-psychiatry: citum '2017: 7: Vaswani A...: vol. 30: pp. 5998-6008: available at' vs oracle '7Vaswani A... 2017;30:599'. Also proceedings-of-the-estonian-academy-of-sciences-numeric (17/53). One bounded migrate-research pass.

## Summary of Changes

Root cause was threefold in the XML template compiler path:
1. assign_layout_order only numbered macro-call nodes, so direct layout nodes (citation-number, title, volume, page) had no source_order and sorted after every macro-derived component.
2. Layout-level macro children never inherited the macro's order (only nested macros got the fill-in), so e.g. the author Names node sorted last.
3. collect_condition_occurrences reset a type-less branch to Default context, so a variable condition nested inside a type condition (if variable=URL inside if type=webpage) leaked its components (available at, url, accessed) unsuppressed into every entry.

Additionally the merge took base component shape and position from the earliest occurrence in ANY branch; it now prefers the default-branch occurrence (order and shape, e.g. bare volume instead of the book branch's labeled volume).

Fixes in crates/citum-migrate/src/lib.rs (assign_layout_order numbers all renderable nodes; layout macro children inherit the call-site order) and crates/citum-migrate/src/template_compiler/compilation.rs (context threading through nested conditions; default-context preference in merge_occurrences). Regression tests: crates/citum-migrate/tests/bibliography_layout_order.rs.

Evidence (strict --force-migrate oracle, citations+bibliography): brazilian-journal-of-psychiatry 47/58 -> 53/58 (91.4%); proceedings-of-the-estonian-academy-of-sciences-numeric -> 56/58; zeitschrift-fur-allgemeinmedizin 52/58 -> 58/58; scientia-iranica -> 54/58. Sentinels hold: apa 57/57, chicago-author-date 57/57, nature 54/58. Full gate clean, 1554/1554.
