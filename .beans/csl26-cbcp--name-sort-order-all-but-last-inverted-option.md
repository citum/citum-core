---
# csl26-cbcp
title: 'Name-sort-order: all-but-last-inverted option'
status: todo
type: task
priority: low
tags:
    - schema
    - sorting
    - contributors
created_at: 2026-07-12T15:36:48Z
updated_at: 2026-07-12T18:14:18Z
parent: csl26-kcda
---

CSL schema#134: at least two journals invert all names in an author list
except the last — [Bioscene](http://www.acube.org/bioscene/submission-guidelines/)
and the [African Journal of Food, Agriculture, Nutrition and Development](http://www.ajfand.net/AJFAND/informationtoauthors.html):

> 1. Gardner MN, Halweil AO and JM Nono
> 2. Spencer N Confirmed NameSortOrder
(style.json) has only {family-given, given-family} — a single-name display
direction, not this list-level pattern. NameOrder (a different def) has
{given-first, family-first, family-first-only} — also not a match; that's
about which contributor in a list is inverted (first only), not "all but
the last." Standalone, small — doesn't fit cleanly into another theme.

- [ ] Design a new list-level name-order variant (e.g. all-but-last-inverted)
