---
# csl26-mmkn
title: 'de-CNE: rename files + original-script view key'
status: completed
type: task
priority: normal
created_at: 2026-06-10T11:51:45Z
updated_at: 2026-06-10T12:00:43Z
---

Round-3: rename -cne files to behavior-descriptive names, drop all CNE prose/comments, rename MultilingualView::Original → OriginalScript (serde: original-script). Covers both PR A (citum-core #899) and PR B (citum-org #21).

## Summary of Changes

- Renamed `MultilingualView::Original` → `OriginalScript` (serde: `original-script`)
  across schema, presets, engine, and all tests.
- git mv: chicago-notes-18th-cne.yaml → chicago-notes-18th-script.yaml,
  multilingual-cne-refs.yaml → multilingual-east-asian-refs.yaml,
  multilingual-cne-chicago.yaml → multilingual-east-asian-chicago.yaml.
- Fixture item IDs and engine test fn names de-cne'd.
- All CNE prose/comments replaced with behavior descriptions.
- Schema regenerated (original → original-script const).
- citum-org post updated: view key, bash paths, CNE sentence removed.
- 1542/1542 tests pass; render output identical.
