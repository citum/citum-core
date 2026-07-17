---
# csl26-yhnr
title: Preserve local vs container editor provenance for author substitution
status: todo
type: bug
priority: normal
tags:
    - engine
    - conversion
    - contributors
    - fidelity
created_at: 2026-07-17T13:23:52Z
updated_at: 2026-07-17T13:23:52Z
---

CollectionComponent currently exposes its enclosing collection editor through the generic editor() accessor. The GB/T author-substitution chain therefore promotes a container editor into an authorless chapter's primary author slot, duplicating it before and after //.

Implement a provenance-aware local-editor lookup (or equivalent substitution path) so author substitution considers only contributors belonging to the component itself, while normal container/editor rendering continues to resolve the enclosing collection editor. Add conversion and rendering regressions covering both an editorless chapter with a container editor and a chapter with its own editor.
