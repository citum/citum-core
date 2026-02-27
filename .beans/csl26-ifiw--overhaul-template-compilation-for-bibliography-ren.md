---
# csl26-ifiw
title: Overhaul template compilation for bibliography rendering
status: todo
type: epic
priority: high
created_at: 2026-02-07T18:20:28Z
updated_at: 2026-02-27T01:14:33Z
---

Epic to track remaining template-compilation and post-processing issues for bibliography rendering.

Historical baseline at creation time showed broad failures, but current migration quality is substantially improved. As of 2026-02-27:

- Top-10 aggregate: bibliography 100% for `7/10` styles
- Primary observed component issue: `publisher:extra`

This epic should now focus on residual style-level mismatches (especially `chicago-author-date`, `nature`, and `cell`) and concrete pass-level regressions in `citum-migrate` (`template_compiler` and `passes/*`), rather than broad "ordering is broken everywhere" assumptions.
