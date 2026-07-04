---
# csl26-3m45
title: Render EDTF seasons via locale season terms
status: todo
type: task
tags:
    - dates
    - localization
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

EDTF seasons (2023-21 = Spring 2023) silently render as bare years: Edtf::month() returns None for seasons and no engine code reads locale.dates.seasons despite en-US shipping four season names. Map seasons 21-24 through locale.dates.seasons wherever months resolve, or emit a structured warning while unsupported. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 6.
