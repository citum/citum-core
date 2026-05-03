---
# csl26-wz7v
title: Remove legacy annotation styling options
status: completed
type: cleanup
priority: normal
tags:
    - engine
    - cli
created_at: 2026-05-02T23:55:00Z
updated_at: 2026-05-02T23:55:00Z
---

Follow-up to the structural annotation refactor: remove the remaining legacy CLI 
styling flags `--annotation-italic` and `--annotation-break` and their 
corresponding fields in `AnnotationStyle`.

These are presentation concerns that should be handled via the output format's 
structural rendering (e.g. CSS for HTML italics, or document-level paragraph 
styling for LaTeX/Typst).
