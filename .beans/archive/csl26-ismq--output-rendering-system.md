---
# csl26-ismq
title: Output & Rendering System
status: completed
type: epic
priority: high
created_at: 2026-02-07T12:12:16Z
updated_at: 2026-03-09T15:30:00Z
blocking:
    - csl26-li63
---

Pluggable output formats and document processing integration completed for the
current rendering architecture.

Goals:
- [x] Abstract renderer trait for multiple output formats
- [x] Implement HTML renderer with semantic classes
- [x] Implement Djot renderer with clean markup
- [x] Implement LaTeX renderer with native escaping
- [x] Implement Typst renderer
- [x] Support full document processing (citations in context)

Architecture:
- Trait-based design allows easy format addition
- Semantic markup (csln-title, csln-author, etc.)
- Clean separation: processor → renderer → output

Integration:
- Works with batch mode (CLI/Pandoc)
- Works with server mode (real-time processing)
- Supports round-trip editing (preserve structure)

Refs: csln#105 (pluggable renderers), csln#86 (Djot),docs/architecture/PRIOR_ART.md (citeproc-rs/jotdown)

Exit criteria met:
- Output renderer trait foundation landed on `main`.
- HTML, Djot, LaTeX, and Typst renderers landed on `main`.
- Full document processing support landed on `main`.
- Remaining rendering follow-on work now lives in narrower beans such as
  `csl26-9a89` and `csl26-k7nf`.

## Summary of Changes

- Closed this epic because all linked rendering-system child beans are
  completed: `csl26-qom9`, `csl26-4eyw`, `csl26-q8zt`, and `csl26-93yh`.
- Marked Typst renderer support complete to match current engine, CLI, and
  server behavior.
- Re-scoped the epic as a completed architecture milestone, with follow-on
  edge-case rendering work left to narrower beans.
