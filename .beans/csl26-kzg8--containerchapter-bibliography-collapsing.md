---
# csl26-kzg8
title: Container/chapter bibliography collapsing
status: todo
type: feature
priority: low
tags:
    - schema
    - rendering
created_at: 2026-07-12T15:36:06Z
updated_at: 2026-07-12T16:02:13Z
parent: csl26-kcda
---

No general mechanism to collapse multiple chapters/contributions from the
same container into one shared bibliography entry (with individually
citable chapters), for the non-anonymous case:
- CSL schema#370 (shorten repeated-container references)
- CSL schema#436 (single bibliography entry for a multi-author edited
  volume, individual chapters cited separately) — same underlying gap

IMPORTANT: #436's proposed solution is a new CSL-XML conditional
(`<if collection="parent">`/`<else-if collection="child">`) plus new
elements and terms — template-language growth Citum's declarative model
does not take on by design. Don't implement that shape.

Related, and the right precedent instead: DIVERGENCE_REGISTER div-010
(bibliography.options.anonymous-entries) implements container-led
reordering via a declarative style-level option, not a template
conditional, for the anonymous-author case. This bean is about
generalizing that same *kind* of solution (an option, not a conditional)
to the non-anonymous, explicit-style-choice case #370/#436 describe.

- [ ] Design: extend div-010's container-led mechanism, or a new option
- [ ] Confirm relationship to div-010 in DIVERGENCE_REGISTER when scoping
