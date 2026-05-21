---
# csl26-dqtx
title: Reference corpus exposes APA 6 vs 7 semantic differences
status: completed
type: bug
priority: critical
created_at: 2026-05-21T00:14:54Z
updated_at: 2026-05-21T00:49:40Z
parent: csl26-f1u7
blocking:
    - csl26-tjqn
    - csl26-ly8d
---

PR #767 (csl26-39tm) shipped a 5-line minimized apa-6th-edition wrapper because the oracle reference corpus could not distinguish apa-6th from apa-7th rendering. That is a **corpus gap**, not a genuine equivalence: APA 6th and APA 7th differ in known ways the standard fixtures do not exercise.

## Known semantic differences not in the corpus

- **et al. threshold:** APA 6 abbreviates at 6+ authors; APA 7 at 3+ (citation form).
- **Bibliography author cap:** APA 6 lists up to 7 then uses ellipsis-and-last; APA 7 lists up to 20.
- **Ampersand vs and:** APA 6 uses `&` in narrative citations and references; APA 7 uses `&` in parenthetical/references but `and` in narrative.
- **Publisher location:** APA 6 includes city; APA 7 omits.
- **DOI formatting:** APA 6 `doi:` or `http://dx.doi.org/`; APA 7 `https://doi.org/`.
- **Et al. on first vs subsequent cite of 3+ authors:** APA 6 spells out on first, et al. after; APA 7 et al. always.

## Why this matters

The minimized form for apa-6th is currently structurally identical to declaring it an alias of apa-7th — the very thing csl26-39tm explicitly prohibited ("without treating APA 6th as an alias of APA 7th"). The oracle gate did not catch this because the fixtures do not include references that would exercise the rules above (e.g., no 4-author reference, no narrative citation, no early-page DOI in the deprecated format).

## Scope

Extend `tests/fixtures/references-expanded.json` and `tests/fixtures/citations-expanded.json` to include:

- A reference with exactly 4 authors (exercises citation et al. threshold differences).
- A reference with 8+ authors (exercises bibliography author cap difference).
- A narrative citation form (`@author-narrative` style) for a 2-author work (ampersand vs and).
- A reference with a DOI (exercises formatting rule).
- A book reference with publisher location (city should appear in APA 6, not APA 7).

Re-run the SQI scorecard. Expected outcomes:

1. apa-6th-edition minimized form *fails* oracle equivalence on the expanded corpus.
2. The scorecard's accept-minimized gate rejects the 5-LOC form (correct: real differences exist).
3. The compression candidates table reports apa-6th as rejected, with the specific failing references identified.

Once the corpus exposes real differences, csl26-tjqn (default minimize) and csl26-ly8d (broader minimize) become safe to land — their oracle gates will correctly reject false positives.

## Related

- Parent: csl26-f1u7
- Blocks: csl26-tjqn (cannot make auto-minimize the default until the oracle gate is trustworthy)
- Blocks: csl26-ly8d (same reason — extending minimize to more parents amplifies the false-positive risk)
- Reverses (partially): the apa-6th-edition row in `docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md` and the corresponding compression-accepted ✓ in the scorecard. Once the corpus is expanded, the row should flip back to standalone (5,661 LOC) with the gap visibility coming from the *fixture coverage* rather than a fake 5-LOC win.



## Resolution

Implemented strict normalized-output minimization acceptance and expanded clustered citation coverage. APA 6 is rejected as an unsafe apa-7th wrapper candidate by scorecard evidence; no-flag migration remains standalone. The remaining 5,661-line standalone output is tracked separately as converter bloat in csl26-kd28.
