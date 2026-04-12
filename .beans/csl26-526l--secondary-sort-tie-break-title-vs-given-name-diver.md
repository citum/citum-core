---
# csl26-526l
title: 'secondary sort tie-break: title vs given-name divergence from citeproc-js'
status: todo
type: bug
created_at: 2026-04-12T21:10:05Z
updated_at: 2026-04-12T21:10:05Z
---

For alphabetically-sorted bibliographies, when two items share the same first-author family name, Citum uses title as the secondary sort key while citeproc-js uses given-name. This produces different numeric citation labels in numeric styles, causing citation fidelity failures that cannot be attributed to div-004 (which only covers anonymous items). First observed in ACM SIG Proceedings fixture: ITEM-10 (Smith, Jane — 'Machine Learning...') and ITEM-28 (Smith, Patricia — 'Discussion on Citum Schema Design') swap positions — oracle puts Jane before Patricia (given-name sort J<P), Citum puts Discussion before Machine Learning (title sort D<M). Because the two effects interact with div-004 in the same fixture, the divergence-aware adjustment cannot fire cleanly. Needs: (1) investigate what sort key(s) citeproc-js uses after author family name, (2) decide whether to match citeproc behavior or register as a divergence, (3) if fixed, re-verify numeric styles.
