---
# csl26-zzun
title: 'engine: fix multi-author name order in non-first positions'
status: completed
type: bug
priority: normal
created_at: 2026-04-11T11:34:13Z
updated_at: 2026-04-11T18:22:15Z
---

In bibliography multi-author contributor lists, non-first authors are being rendered family-first when they should be given-first (e.g. 'Grene, David, and Lattimore, Richmond' vs 'Grene, David, and Richmond Lattimore'). This is a known gap affecting chicago-zotero-bibliography and likely other styles. name-order: family-first should only apply to the first contributor in a list.

## Summary of Changes

- Added `NameOrder::FamilyFirstOnly` variant to schema (`citum-schema-style/src/template.rs`)
- Engine handles it in `format_single_name`: `index == 0` determines inversion (`values/contributor/names.rs`)
- Updated 7 styles to `name-order: family-first-only`: `chicago-shortened-notes-bibliography`, its classic-archive variant, `chicago-notes-bibliography-17th-edition`, `chicago-notes-bibliography-classic-archive-place-first-no-url`, and all 3 `new-harts-rules-notes` variants
- Closed div-006 in the divergence register
- Schema bump: patch (0.29.1 → 0.29.2, additive enum variant)
