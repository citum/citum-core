---
# csl26-uzkj
title: 'Spec: bidi/RTL handling in rendered output'
status: todo
type: task
tags:
    - multilingual
    - rendering
created_at: 2026-07-18T20:32:33Z
updated_at: 2026-07-18T20:32:33Z
parent: csl26-0ugp
---

The engine has no directionality model: Arabic/Hebrew bibliography entries mixing RTL text with Latin DOIs, numbers, and publisher names will visually scramble in plain-text output. Write a spec for format-aware bidi handling: dir attributes or bdi elements in HTML, FSI/PDI isolates at field boundaries in plain text, applied where a field's script direction differs from context. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(d).
