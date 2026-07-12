---
# csl26-v3g2
title: 'Data-model additions: social handle, multiple URLs'
status: todo
type: feature
priority: low
tags:
    - schema
created_at: 2026-07-12T15:36:17Z
updated_at: 2026-07-12T16:02:31Z
parent: csl26-kcda
---

Two confirmed-absent data fields on InputReference:
- Social-media handle/username (for citing tweets, Instagram, TikTok posts)
  — CSL schema#432. Verified `platform` (software platform: Windows/macOS/
  Linux) and `network` (broadcaster/streaming network) fields do NOT cover
  this; no handle/username field exists anywhere in the schema.
- Multiple URLs for one reference (canonical + archival/mirror copies) —
  CSL schema#462. `url` is confirmed single-valued (`["string","null"]`
  format uri). Note: `archive`/`archive-info`/`archive-location` fields
  already cover the "cite the archived copy separately" case reasonably
  well (see #387 in the audit, already addressed) — worth checking during
  design whether #462 needs more than what archive-location already gives.

- [ ] Design: dedicated handle/username field, likely with a platform enum
- [ ] Design: multi-URL — array field vs. relying on archive-location for
      secondary copies (avoid solving something already covered)
