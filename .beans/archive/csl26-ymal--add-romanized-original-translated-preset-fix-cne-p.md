---
# csl26-ymal
title: Add romanized-original-translated preset + fix CNE prose
status: completed
type: feature
priority: normal
created_at: 2026-06-10T11:21:35Z
updated_at: 2026-06-10T11:30:49Z
---

Two-part change: (1) add RomanizedOriginalTranslated preset to MultilingualPreset in citum-core (romanized+original+[translated] titles, romanized+original names, Latn+CJK native ordering baked in), collapsing the cne style to a one-liner; (2) fix misleading CNE framing in the citum-org multilingual news post.

## Summary of Changes

- Added  variant to  in : resolves to , , , Han/Hangul native ordering baked in.
- Added corresponding test in  (278/278 pass, full suite 1542/1542).
- Updated  §2.1 preset table and note.
- Collapsed  explicit block to one-liner  (render output identical).
- Regenerated  (new enum const only).
- citum-org prose fix deferred to PR B.
