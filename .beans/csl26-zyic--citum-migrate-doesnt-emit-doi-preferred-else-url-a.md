---
# csl26-zyic
title: citum-migrate doesn't emit DOI-preferred-else-URL access fallback
status: todo
type: task
priority: high
created_at: 2026-09-10T13:48:18Z
updated_at: 2026-09-10T13:48:18Z
parent: csl26-ccdt
---

The RENDER_WHEN_DISPOSITION audit's central claim -- a CSL <choose><if variable=DOI>...<else-if variable=URL>...</choose> fallback can't be emitted by citum-migrate because render-when is closed to new migration targets -- is confirmed corpus-wide, not just theoretical.

Verified: american-society-of-mechanical-engineers.yaml had this exact gap (both url and doi rendered unconditionally, no access phrase or accessed-date bracket at all). Fixed by hand in PR #1273 as the worked example: select: first between {variable: doi} and a group rendering '[Online]. Available: {url}. [Accessed: {date}]', plus a new DateForm::DayMonthAbbrYearHyphen for the '15-Jan-2024' bracket shape, plus entry-suffix-after-url/doi:true (the engine suppresses the trailing entry period by default when an entry ends in a bare URL/DOI; ASME's real CSL does not). Result: 5->11/67 exact-parity rows, zero regressions in the 35-style corpus.

Grep signature used to find candidates: a style YAML has both 'variable: url' and 'variable: doi' present, with zero 'render-when'/'select: first' occurrences anywhere in the file (meaning they're almost certainly unconditional siblings, not gated). That signature currently also matches (verified against the real .csl for ieee/apa/harvard-cite-them-right that they truly use the DOI-else-URL <choose> idiom; the rest are grep hits only, not yet individually confirmed):

- styles/embedded/ieee.yaml (ASME's own base -- different access-macro shape than ASME's, not yet designed)
- styles/embedded/apa-7th.yaml (high-traffic style, needs care)
- styles/american-chemical-society.yaml
- styles/embedded/american-medical-association.yaml
- styles/american-medical-association-alphabetical.yaml
- styles/chicago-notes-bibliography-17th-edition.yaml
- styles/entomological-society-of-america.yaml
- styles/embedded/gb-t-7714-2025-author-date.yaml
- styles/embedded/gb-t-7714-2025-note.yaml
- styles/harvard-cite-them-right.yaml
- styles/mhra-notes.yaml
- styles/embedded/modern-language-association.yaml
- styles/numeric-comp.yaml
- styles/royal-society-of-chemistry.yaml

(chicago-author-date-18th.yaml and chicago-notes-18th.yaml also matched the grep but are excluded here -- 8 Chicago beans are in-flight, per the audit's own caution.)

Two possible fixes, needing a decision before more style-by-style hand patches:
1. Keep hand-patching each style's YAML individually (what ASME did) -- correct per-style but doesn't fix the root cause, so every future re-migration from CSL for these styles regresses the fix.
2. Extend citum-migrate itself to recognize this specific <choose><if DOI>...<else-if URL>...</choose> idiom and emit a select: first group directly. This is the real fix but is a converter change, which per CLAUDE.md needs its own spec in docs/specs/ before implementation -- not done in this bean.

Each remaining style needs the same individual verification PR #1273 did for ASME (real .csl idiom confirmed, honest post-fix ceiling checked against fixture residue, not just the grep signature) before assuming the fix applies -- some fraction of these 14 will have secondary unrelated bugs (title-case, quote-routing, missing fields) that cap how many rows can actually flip, exactly like ASME's own remaining 36 failures.
