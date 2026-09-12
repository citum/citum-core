---
# csl26-esq8
title: Document parser syntax parity and warnings
status: todo
type: task
priority: normal
tags:
    - parser
    - warnings
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-09-12T13:43:17Z
parent: csl26-8m2p
---

Djot and Markdown parsers disagree: key charsets differ (djot [@smith:2020] silently cites 'smith'), djot lacks prefix/suffix support, markdown drops whole bracket clusters mixing suppress-author states, both parse citations inside code blocks/spans, and markdown's bare textual form @key. greedily eats trailing punctuation into the key (found during F1 fix). Unify the key charset, per-item suppression, mask code ranges via the existing pulldown-cmark/jotdown events, and emit document warnings for dropped/malformed citation candidates. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 15.

## Correction (2026-09-12): do not implement per-item suppression

The scope line above says "unify the key charset, per-item suppression, ..." — that part is
superseded. csl26-zoou (completed) deliberately moved suppress-author from per-item
(`ItemVisibility` on `CitationItem`) to citation-level (`suppress_author` on `Citation`), on the
grounds that mixed-visibility citations have no real authoring use case and CSL 1.0's per-item
design followed Zotero's checkbox UI rather than semantics.

So the parity fix here is the reverse direction: bring the **markdown** parser up to the djot
parser's citation-level behaviour, not push djot toward per-item. The remaining valid items in this
bean are the key charset, the missing affix support, code-range masking, and the warnings.

Locator-vs-item-suffix disambiguation, which affix support forces a decision on, is split out as
csl26-ibb1. See csl26-s2zo for upstream djot#32 alignment.
