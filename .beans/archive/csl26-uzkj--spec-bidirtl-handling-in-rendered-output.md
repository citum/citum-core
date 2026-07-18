---
# csl26-uzkj
title: 'Spec: bidi/RTL handling in rendered output'
status: completed
type: task
priority: normal
tags:
    - multilingual
    - rendering
created_at: 2026-07-18T20:32:33Z
updated_at: 2026-07-18T21:00:15Z
parent: csl26-0ugp
---

The engine has no directionality model: Arabic/Hebrew bibliography entries mixing RTL text with Latin DOIs, numbers, and publisher names will visually scramble in plain-text output. Write a spec for format-aware bidi handling: dir attributes or bdi elements in HTML, FSI/PDI isolates at field boundaries in plain text, applied where a field's script direction differs from context. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(d).

## Summary of Changes

Draft spec delivered: docs/specs/BIDI_OUTPUT.md. Defines the bidi-isolation opt-in (none | fields) on MultilingualConfig, direction detection (metadata evidence via effective script, first-strong content fallback, positive-evidence preserved), per-format realization (HTML bdi + dir attribute; FSI/PDI isolates for the plain-text family; LaTeX/Typst deferred), the no-self-reordering guarantee, and the fixture escaping constraint motivated by the 2026-07-18 alint oss-no-bidi-controls CI failure. Implementation is follow-on work under the spec's acceptance criteria.
