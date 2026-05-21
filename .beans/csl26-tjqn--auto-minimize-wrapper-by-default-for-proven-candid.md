---
# csl26-tjqn
title: Auto-minimize wrapper by default for proven candidates
status: todo
type: feature
priority: high
created_at: 2026-05-21T00:10:10Z
updated_at: 2026-05-21T00:10:24Z
parent: csl26-f1u7
---

PR #767 (csl26-39tm) delivered the `--minimize-wrapper` mechanism but kept it opt-in: `citum-migrate styles-legacy/apa-6th-edition.csl` with no flags still outputs the 5,661-line standalone form. The 5-LOC minimized form only appears when the caller passes `--family-candidate auto --minimize-wrapper` (the scorecard does this; users invoking the binary directly do not).

Result: the bean csl26-39tm's literal target ("reduce migrated output below 1,500 LOC") is met as a *capability* but not as default behavior. End users running the migrate binary see no compression.

## Scope

Make minimization the default outcome when oracle evidence proves it safe. Two viable approaches; pick during planning:

1. **Cached-decision lookup.** The scorecard (or a CI step) writes a checked-in manifest `crates/citum-migrate/data/minimization-decisions.yaml` listing legacy CSL ids whose minimized form has been oracle-verified equivalent to standalone. The binary loads this manifest and auto-promotes the candidate parent when migrating a listed style. Lightweight; keeps oracle out of the binary.

2. **In-binary oracle gate.** Add citum-engine as a dep, embed a fixed reference corpus (`tests/fixtures/references-expanded.json` + `citations-expanded.json`), render both standalone and wrapper-minimized forms internally, decide. Heavier; binary size grows; but no manifest drift risk.

## Acceptance

- `cargo run --bin citum-migrate -- styles-legacy/apa-6th-edition.csl` with no flags emits the 5-LOC minimized form.
- Oracle citation pass >= 18/18 and bibliography pass >= 33/34 on the minimized form.
- No regression for any existing sentinel in the SQI scorecard.
- Default behavior for legacy CSL ids with no proven compression candidate is unchanged.
- Decision audit trail: the evidence sidecar reports whether the chosen form came from the cached manifest, in-binary oracle gate, or the caller's explicit flags.

## Related

- Parent epic: csl26-f1u7
- Sibling: csl26-ly8d (extend minimize to template-link / independent-parent-link parents) — both should land before csl26-f1u7 closes.
- Original capability bean: csl26-39tm (completed, archived; capability shipped but default behavior gap left open)
