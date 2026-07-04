---
# csl26-o33x
title: Thread locale quote terms through rendering
status: todo
type: task
tags:
    - localization
    - rendering
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

Locale open/close/inner quote terms exist in the schema (en-US and others) but every backend hardcodes English marks (unicode_quote_marks, per-backend quote()/wrap_punctuation, LaTeX backtick pairs). A fr-FR style declaring guillemets renders curly quotes everywhere. Thread the active locale quote terms through quote_marks(depth); keep hardcoding only as fallback. Needs locale access at the OutputFormat boundary — design first. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 12.
