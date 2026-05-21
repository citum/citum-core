---
# csl26-atyp
title: Long-term style edition + family strategy spec
status: completed
type: task
priority: normal
created_at: 2026-05-21T16:21:47Z
updated_at: 2026-05-21T16:24:06Z
---

Create docs/specs/STYLE_EDITIONS_AND_FAMILIES.md codifying policy for how Citum handles style editions and families over the long term: inheritance shape (siblings under archetype), embedding retention (N + N-1), retirement workflow, version field policy, and tooling impact on citum-migrate / style skills. Pure docs PR; mechanism changes are deferred to follow-up beans.

## Summary of Changes

- Created `docs/specs/STYLE_EDITIONS_AND_FAMILIES.md` (v1.0, Active) codifying long-term policy on style editions and families.
- §1 Edition inheritance shape: siblings under optional hidden archetype; no edition-on-edition stacking.
- §2 Embedding retention: current edition + immediately prior edition (N + N−1); older editions move to filesystem or are removed.
- §3 Retirement workflow: single-PR coordinated rewrites with CI integrity check; `deprecated:` marker + SoftDegrade warnings during deprecation window.
- §4 Version-field policy: no new style version field. Keep `StyleBase` key (structured identity), `info.edition` (display), `info.citum_version` (engine requirement). Reject `info.version`.
- §5 Tooling impact: documented (not implemented). Follow-up beans tracked in §7.
- Cross-referenced from `STYLE_PRESET_ARCHITECTURE.md`, `STYLE_TAXONOMY.md`, `FORWARD_COMPATIBILITY.md`.

Pure docs PR; no Rust changes; no schema changes.
