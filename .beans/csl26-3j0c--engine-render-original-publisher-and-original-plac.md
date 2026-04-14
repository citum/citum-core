---
# csl26-3j0c
title: 'engine: render original-publisher and original-place for reprints'
status: in-progress
type: feature
priority: normal
created_at: 2026-04-11T11:34:06Z
updated_at: 2026-04-14T12:55:41Z
---

Chicago 18th §14.16 reprint pattern: '(original-year) current-year. Title. Original Publisher. Note. Current Publisher'. CSL fields original-publisher and original-publisher-place are not yet mapped from CSL JSON to Citum's original WorkRelation, so they silently drop. 7 chicago-zotero-bibliography benchmark items fail on this pattern (date already fixed by csl26-tpmn engine fix, but original-publisher and edition text still differ).



## Checklist

- [x] Map original-publisher and original-publisher-place into the embedded original relation.
- [x] Expose original publisher/place as template variables and render them in the engine.
- [x] Add Chicago notes reprint rendering and regression coverage.
- [x] Run full Rust verification and regenerate schemas if needed.
- [ ] Run bean hygiene and open the PR.
