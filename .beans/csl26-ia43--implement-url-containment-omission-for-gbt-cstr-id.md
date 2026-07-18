---
# csl26-ia43
title: Implement URL-containment omission for GB/T CSTR identifier tail
status: draft
type: task
priority: normal
tags:
    - style
    - migrate
    - fidelity
created_at: 2026-07-18T11:41:22Z
updated_at: 2026-07-18T11:41:30Z
parent: csl26-8uxa
---

GB/T 7714 §7.9.1/§7.9.2 may require omitting the identifier: cstr rendering tail when the reference URL already contains the same identifier, regardless of the tex.cstr/CSTR alias question that div-009 currently documents. Raised by reviewer @YDX-2147483647 on PR #1064. Blocked on reviewer confirmation of the interpretation before implementing.
