---
# csl26-1mkv
title: Fix duplicate sort keys in 16 styles
status: todo
type: bug
created_at: 2026-02-27T22:58:35Z
updated_at: 2026-02-27T22:58:35Z
---

16 styles were skipped by scripts/apply-sort-presets.py due to duplicate sort keys (migration artifacts). Each has repeated keys like author/author/issued instead of a clean sequence. Fix by deduplicating and replacing with the appropriate SortPreset.\n\nStyles to fix:\n- annual-reviews-author-date.yaml\n- begell-house-chicago-author-date.yaml\n- elsevier-vancouver-author-date.yaml\n- mhra-author-date-publisher-place.yaml\n- mhra-notes.yaml\n- mhra-shortened-notes-publisher-place.yaml\n- modern-language-association.yaml\n- museum-national-dhistoire-naturelle.yaml\n- new-harts-rules-author-date-space-publisher.yaml\n- oscola-no-ibid.yaml\n- oscola.yaml\n- pensoft-journals.yaml\n- sage-harvard.yaml\n- springer-basic-author-date-no-et-al.yaml\n- the-company-of-biologists.yaml\n- the-geological-society-of-london.yaml\n\nFor each: inspect the sort block, determine the correct canonical sequence, then apply the matching SortPreset (or author-date-title if ambiguous). Extend apply-sort-presets.py to handle deduplication if appropriate.
