---
# csl26-ifiw
title: Track residual XML bibliography migration deltas
status: completed
type: epic
priority: high
created_at: 2026-02-07T18:20:28Z
updated_at: 2026-02-28T00:20:51Z
---

Epic to track the remaining XML-path bibliography and citation deltas after the
large early template-compilation failures were resolved.

Historical baseline at creation time showed broad failures, but this is no
longer an overhaul-sized problem. As of 2026-02-28 verification, the explicit
top-10 cohort is now fully green:

- Citations: `10/10` styles at `13/13`
- Bibliography: `10/10` styles at full match
- `nature` report `publisher:extra` is fixed
- `chicago-author-date` citation disambiguation and bibliography gaps are fixed
- `cell` dataset/patent/personal-communication bibliography gaps are fixed

This epic is complete. Future regressions in `citum-migrate`
(`template_compiler` and `passes/*`) should be tracked as new concrete bugs,
not by re-opening the earlier broad "ordering is broken everywhere" problem
statement.
