---
# csl26-s2zo
title: Align djot citation syntax with djot#32 outcome
status: todo
type: epic
priority: normal
tags:
    - citation
    - parser
    - forward-compat
created_at: 2026-09-12T13:42:18Z
updated_at: 2026-09-12T13:42:18Z
---

Upstream jgm/djot#32 is converging on a citation syntax. Our djot parser
(`crates/citum-engine/src/processor/document/djot/parsing.rs`) implements an earlier draft.
This epic tracks the remaining deltas so they are decided together rather than drifting.

Upstream: https://github.com/jgm/djot/issues/32 (jgm's restated proposal, 2026-09-12)

## Already aligned, no work needed

- **Bracket-only citations.** `find_citations` only scans for `[`, and `parse_parenthetical_citation`
  requires the closing `]`. jgm's stated position is full bracket encapsulation for unambiguous
  parsing. Bare/unbracketed keys are not supported here and should not be.
- **Citation-level modifier scope.** `parse_citation_content` parses `+`/`-` per item but promotes
  them to `citation.mode` / `citation.suppress_author`. This matches the completed decision in
  csl26-zoou and is the position argued upstream.

## Covered elsewhere

- Key charset and missing affix support: csl26-esq8.
- Document-level nocite API: csl26-cpb7 (completed) — inline *syntax* is not covered there.
