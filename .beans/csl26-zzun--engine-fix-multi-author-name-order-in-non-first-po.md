---
# csl26-zzun
title: 'engine: fix multi-author name order in non-first positions'
status: todo
type: bug
priority: normal
created_at: 2026-04-11T11:34:13Z
updated_at: 2026-04-11T11:34:13Z
---

In bibliography multi-author contributor lists, non-first authors are being rendered family-first when they should be given-first (e.g. 'Grene, David, and Lattimore, Richmond' vs 'Grene, David, and Richmond Lattimore'). This is a known gap affecting chicago-zotero-bibliography and likely other styles. name-order: family-first should only apply to the first contributor in a list.
