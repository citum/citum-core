---
# csl26-28ag
title: T&F trio structural conformance
status: completed
type: feature
priority: normal
created_at: 2026-06-21T10:49:22Z
updated_at: 2026-06-21T11:49:27Z
---

Finish structural conformance for the three embedded Taylor & Francis styles against the official PDFs (local-only at /tmp/tf/; not in-repo). Detail per-style in docs/architecture/audits/2026-06-20_STYLE_GUIDE_CONFORMANCE.md (T&F section).

- NLM (tf_nlm.pdf): journal needs the `Year;Vol(Iss):pages` regroup and book/chapter `Place: Publisher; Year` reorder (same group shape as the AMA journal group). Author period + chapter `In: <names>, editors. <Book>` already landed in PR #946.
- CSE / Style C (tf_c.pdf): chapter `In:`/editors structure and journal punctuation (`37, (1); 1–13` -> `37(1):1–13`), author period.
- Chicago / Style F (tf_f.pdf): chapter title period-in-quote, redundant `(eds.)`, `: 683–703:` colons, journal `(year)`.

DECISION NEEDED (bucket B): T&F Style F prescribes sentence-case, UNQUOTED article titles; both Citum and the citeproc CSL render Title Case in quotes. Adopting the guide is an intentional divergence from the CSL reference — needs explicit sign-off before implementing.

Follow-up from the guide-conformance sweep (csl26-53zy / PR #946).


## Summary of Changes

Closed the YAML-addressable structural conformance gaps for the three embedded
T&F styles, verified against the official PDFs (`/tmp/tf/`).

**NLM (`...national-library-of-medicine-core.yaml`)** — rewrote bibliography
type-variants with delimiter-based grouping:
- Journal regroup → `Journal. Year;Vol(Iss):pages.` (e.g. `Nature. 2015;521:436–444.`).
- Book/chapter `Place: Publisher; Year` reorder; `p.` (not `pp.`) page label.
- Chapter `In: Editors, editors. Book. Place: Publisher; Year. p. pages.` now
  byte-matches the `tf_nlm.pdf` sample; conference keeps its proceedings serial.

**CSE / Style C (`...council-of-science-editors-author-date-core.yaml`)** —
- Author period (`Adams J. 1797.`), journal `Vol(Iss):pages` (`37(1):1–13`),
- Chapter `In: Editors, editors. Book. Place: Publisher. p. pages` (replaced the
  `on edited by` migration bug), book `Place: Publisher` reorder.

**Chicago / Style F (`...chicago-author-date-core.yaml`)** — full guide adoption
decision implemented:
- Article (`component`) titles now **sentence-case + unquoted** per `tf_f.pdf`.
- Redundant `(eds.)` removed (guide forbids `eds.` in that position) via
  `role.omit: [editor]`; chapter `, 683–703. Place: Publisher` colon fix.

## Deferred to csl26-maim (engine, not YAML)

- [ ] **Sentence-case proper-noun preservation.** Style F applies sentence-case to
  article titles only; `monograph`/`container-monograph` titles stay title-case
  because the engine's `TextCase::Sentence` lowercases proper nouns
  (`Cambridge`→`cambridge`). Extend Style F to book titles once the proper-noun
  fix lands. *Do not lose track of this.*
- Style F encyclopedia/`default` fallback double-colon (hits the generic base
  template; out of scope for the trio's flagged types).
