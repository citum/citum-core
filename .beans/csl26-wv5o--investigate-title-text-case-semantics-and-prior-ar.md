---
# csl26-wv5o
title: Investigate title text-case semantics and prior art
status: todo
type: feature
priority: normal
created_at: 2026-03-09T21:07:35Z
updated_at: 2026-03-09T21:07:35Z
---

Follow-up to `csl26-suz3`.

Bounded Phase 3 for Djot title markup landed under the current rendering model,
but the broader title/text-case problem remains open and should not be solved
implicitly inside rich-text work.

Requirements:
- review CSL and biblatex prior art for title-case, sentence-case, and
  `.nocase` / case-protection behavior
- decide whether Citum should model general title/text-case semantics in the
  engine, schema, presets, or a combination
- revisit the existing assumption that input data should be normalized toward
  sentence case before any title-case transformation
- define how language-sensitive title formatting interacts with field language,
  multilingual titles, and title/category presets
- produce a spec before implementation if the result changes schema or engine
  behavior materially

Non-goals:
- re-opening the bounded Djot title-markup work already landed in `csl26-suz3`
- shipping `.nocase` or title-case logic without prior-art review and a spec
