---
# csl26-b0ud
title: Synthesized type-variants anchors match no component (processor error)
status: todo
type: bug
priority: high
tags:
    - migrate
    - fidelity
created_at: 2026-07-17T17:53:25Z
updated_at: 2026-07-17T17:53:38Z
---

17 of 66 near-clone pairs in the 2026-07-17 delta-derivability measurement errored because the STANDALONE synthesized migration output fails in the processor: 'template variant operation in bibliography.type-variants[...] matched no component' or TemplateVariantAnchorNotFound. Affected anchors observed: paper-conference, patent, thesis, article, entry-encyclopedia+manuscript, entry-dictionary, review-book. The converter emits type-variant diff operations whose anchor component does not exist in the assembled base template — every ordinary migration of these styles produces a style that panics at render. Repro candidates: freshwater-crayfish, preslia, antarctic-science, china-information, chicago-author-date-classic. Context: docs/architecture/audits/2026-07-17_EXTENDS_DELTA_DERIVABILITY.md
