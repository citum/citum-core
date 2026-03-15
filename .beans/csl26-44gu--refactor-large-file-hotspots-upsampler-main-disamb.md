---
# csl26-44gu
title: Refactor large-file hotspots (upsampler, main, disambiguation)
status: todo
type: task
priority: deferred
created_at: 2026-03-15T16:03:43Z
updated_at: 2026-03-15T16:03:43Z
---

Three files not touched by the simplify wave still exceed 1000 lines and carry scoped clippy suppressions:
- citum-migrate/src/upsampler.rs (1,983 lines)
- citum-migrate/src/main.rs (1,492 lines)
- citum-engine/src/processor/disambiguation.rs (1,022 lines)

Each has a FIXME comment referencing this bean. Address when prioritized.
