---
# csl26-bztv
title: mhra edition/film/interviewer fidelity
status: completed
type: task
priority: high
created_at: 2026-04-17T00:20:10Z
updated_at: 2026-04-17T00:50:38Z
parent: csl26-12hl
---

MHRA bib fails on edition, film genre, and interviewer-host rendering (26/33 bib passing). Style-defect + minor engine.

## Summary of Changes

- Removed global name-form:initials so bibliography uses full given names
- Default template: dropped title quotes for monographs, added translator block, fixed publisher-place to (Place: Publisher) group
- Added type-variants: article-newspaper, broadcast, motion-picture, interview, patent
- Added personal-communication: [] to exclude from bibliography
- Oracle result: 18/18 citations, 32/32 bibliography (was 28/33)
- Portfolio gate passes
