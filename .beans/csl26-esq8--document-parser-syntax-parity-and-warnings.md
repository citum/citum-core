---
# csl26-esq8
title: Document parser syntax parity and warnings
status: todo
type: task
tags:
    - parser
    - warnings
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

Djot and Markdown parsers disagree: key charsets differ (djot [@smith:2020] silently cites 'smith'), djot lacks prefix/suffix support, markdown drops whole bracket clusters mixing suppress-author states, both parse citations inside code blocks/spans, and markdown's bare textual form @key. greedily eats trailing punctuation into the key (found during F1 fix). Unify the key charset, per-item suppression, mask code ranges via the existing pulldown-cmark/jotdown events, and emit document warnings for dropped/malformed citation candidates. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 15.
