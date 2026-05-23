---
# csl26-tjqn
title: Auto-minimize wrapper by default for proven candidates
status: scrapped
type: feature
priority: high
created_at: 2026-05-21T00:10:10Z
updated_at: 2026-05-23T12:21:39Z
parent: csl26-f1u7
---

PR #767 (csl26-39tm) delivered the `--minimize-wrapper` mechanism but kept it opt-in: `citum-migrate styles-legacy/apa-6th-edition.csl` with no flags still outputs the 5,661-line standalone form. The 5-LOC minimized form only appears when the caller passes `--family-candidate auto --minimize-wrapper` (the scorecard does this; users invoking the binary directly do not).

Result: the bean csl26-39tm's literal target ("reduce migrated output below 1,500 LOC") is met as a *capability* but not as default behavior. End users running the migrate binary see no compression.

## Scope

Make minimization the default outcome when oracle evidence proves it safe. Rejected implementation approach:

1. **Cached-decision lookup.** The scorecard (or a CI step) writes a checked-in manifest `crates/citum-migrate/data/minimization-decisions.yaml` listing legacy CSL ids whose minimized form has been oracle-verified equivalent to standalone. This was tried in PR #768 and rejected: loading a source-tree YAML file from the binary is checkout-shaped, distribution-hostile, and masks converter bloat.

Any future design must be distribution-safe and must not make APA 6 default to an APA 7 wrapper unless strict citation and bibliography equivalence proves that safe.

## Acceptance

- No-flag migration emits a minimized wrapper only for candidates with strict citation and bibliography equivalence.
- APA 6 is not minimized to APA 7 unless strict evidence changes.
- No regression for any existing sentinel in the SQI scorecard.
- Default behavior for legacy CSL ids with no proven compression candidate is unchanged.
- Decision audit trail: the evidence sidecar reports whether the chosen form came from default behavior or the caller's explicit flags.

## Related

- Parent epic: csl26-f1u7
- Sibling: csl26-ly8d (extend minimize to template-link / independent-parent-link parents) — both should land before csl26-f1u7 closes.
- Original capability bean: csl26-39tm (completed, archived; capability shipped but default behavior gap left open)



## Current state

Reopened after PR #768 review. A checked YAML manifest loaded by the binary is not an acceptable default-routing mechanism: it is checkout-shaped, distribution-hostile, and risks masking converter bloat. The strict SQI gate now rejects unsafe candidates, but default minimization remains unresolved.

Next work should first reduce known standalone converter bloat (see csl26-kd28) and then revisit any default minimization design with a distribution-safe runtime story.


## Reasons for Scrapping

Bean obsoleted by `csl26-kd28` and current strict-gate behavior; see
`docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md`.

- The cited 5,661-LOC `apa-6th-edition` standalone is now 993 LOC after
  the converter-bloat reduction in `csl26-kd28` (d0f3a80a).
- Across the 18-style sentinel + lab corpus, only `apa-6th-edition` is
  still standalone with a discovered candidate parent; the strict
  equivalence gate added in PR #768 correctly rejects its `apa-7th`
  candidate. Every other candidate is already emitted as an
  ExistingWrapper at lineage time (registry-alias, local-extends).
- The remaining real compression opportunity (`mdpi` via template-link)
  is scoped to `csl26-ly8d`. Default-minimize would change zero observable
  output today and is better revisited as a thin follow-up once
  `csl26-ly8d` lands.
