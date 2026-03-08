---
# csl26-5t6s
title: Note Citation System
status: completed
type: epic
priority: high
created_at: 2026-02-07T07:40:14Z
updated_at: 2026-02-28T00:01:04Z
blocking:
    - csl26-sqsd
---

Implement full note citation support with position tracking (ibid, subsequent), automatic footnote/endnote generation, and note-specific formatting.

## Summary of Changes

Position detection (annotate_positions) and ibid/subsequent spec resolution (resolve_for_position) were already implemented in the engine. Closed by adding ibid/subsequent citation overrides to chicago-notes.yaml and integration tests verifying position-based rendering.
