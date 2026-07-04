---
# csl26-ejaf
title: Djot/Org metacharacter escaping policy
status: todo
type: task
tags:
    - escaping
    - rendering
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

Djot and Org backends emit reference data without escaping their own metacharacters (_, *, ~), mangling formatting when data contains them. Decide and document an explicit policy (escape, or document as accepted limitation) — HTML text escaping was fixed in PR #1002. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 3 (remainder).
