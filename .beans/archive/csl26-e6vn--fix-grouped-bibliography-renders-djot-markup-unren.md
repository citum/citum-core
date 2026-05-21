---
# csl26-e6vn
title: 'fix: grouped bibliography renders Djot markup unrendered in non-plain formats'
status: completed
type: bug
priority: high
created_at: 2026-05-21T19:02:29Z
updated_at: 2026-05-21T20:40:00Z
---

Implemented: grouped bibliography rendering now processes entries through the active output format, so container titles and within-field Djot markup render as HTML/LaTeX/Typst markup instead of leaking PlainText markers.
