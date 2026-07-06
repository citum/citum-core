---
# csl26-21ep
title: 'migrate: compact physics form title suppression'
status: todo
type: bug
priority: low
tags:
    - migrate
    - fidelity
    - style-family
created_at: 2026-06-10T16:56:45Z
updated_at: 2026-07-06T18:56:01Z
parent: csl26-vmcr
---

Cluster C5 (small band, physics family) from docs/architecture/audits/2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md. This is a proven random-sample migration fidelity defect, but not yet proven to be converter-only, engine-side, or schema-driven.

Compact physics styles should suppress article titles and render page-only locators (`T. S. Kuhn, Philosophy of Science 37, 1 (1970)`). Migrated output includes the title and drops the page. Evidence: `springer-physics-author-date` (11/38).

Next step: one bounded migrate-research pass that classifies the root cause before implementation.

## Diagnosis Pointers (2026-07-06 review — hypothesis level, for the bounded classification pass)

From the migrate crate review: the candidate machinery for half of this defect **already exists** — `measured_citation.rs` generates `article-journal-suppress-primary-title` (and title+doi-url combos) in the ArticleJournalSuppression family. Two likely reasons it never wins on the physics band:

1. **The mutations are unpaired.** Compact physics needs title suppression AND page-locator restoration together (`T. S. Kuhn, Philosophy of Science 37, 1 (1970)`). A title-suppression-only candidate still misses the page tokens, so its Jaccard score may not clear candidate_beats against the incumbent; the win condition requires the paired form. The synthesis loop applies one mutation per round and only accepts strict improvements, so it cannot cross a two-defect valley.
2. **The page drop is upstream of selection.** If pages are absent from the seed templates entirely (extractor/compiler drop), no bibliography mutation family reintroduces them — candidates only suppress or reshape existing components (suppress_matching_components sets suppress, nothing adds).

The bounded pass should therefore check, on springer-physics-author-date: (a) does the XML-compiled seed contain the pages component? If not → extractor/compiler locus, fix there. (b) If present, do the suppress candidates score above the incumbent? Instrument with CITUM_MIGRATE_DEBUG_BIB_SELECTION=1. (c) Only if both look right, consider a paired 'compact-physics' patch (suppress title + page-only locator) as one CandidatePatchKind — precedent: the existing multi-matcher ArticleJournalSuppress combos.
