# Output-Driven Template Synthesis

**Status:** Active
**Date:** 2026-06-11
**Bean:** `csl26-aynr`
**Related:** `docs/specs/EMBEDDED_JS_TEMPLATE_INFERENCE.md`, `docs/architecture/audits/2026-06-11_MIGRATE_IMPROVEMENT_WAVE_OUTCOME.md`

## Purpose

Raise `citum-migrate` fidelity by selecting generated templates from rendered
output evidence instead of trusting the structural CSL XML layout compiler.

## Problem

CSL layout XML is procedural. Citum templates are declarative. The existing
migrator can extract CSL metadata and options reliably, but compiling the CSL
layout tree into declarative bibliography and citation templates creates
repeated fidelity defects: component order drift, conditional leakage, lost
affixes, and note-position mismatches.

The converter already has the two engines needed to choose better output:

- the embedded citeproc-js runtime renders the CSL reference output
- `citum-engine` renders candidate Citum styles

The missing mechanism is a deterministic candidate-selection loop that scores
candidate templates against citeproc output before emission.

## Scope

In scope:

- render citeproc-js bibliography and citation reference strings in-process
- render candidate Citum styles with `citum-engine`
- score citation candidates with raw token Jaccard to preserve label behavior
- score bibliography candidates with oracle-compatible normalization, including
  case-only mismatch rejection and greedy entry pairing
- choose the higher-pass candidate, or require a clear similarity margin on
  pass-count ties
- generate a bounded set of typed bibliography patch candidates
- preserve XML extraction for declarative attributes and options

Out of scope for this first implementation:

- LLM-assisted template authoring
- replacing citeproc-js verification scripts
- changing public style schema
- unbounded mutation search
- wrapper minimization policy changes

## Design

The first vertical slice generalizes the measured citation selector into a
bounded measured-candidate selector:

1. Resolve the inferred templates and compile the XML fallback as today.
2. Assemble the normal standalone candidate.
3. Select citation output first, comparing the inferred/current style, XML
   fallback, and bounded citation-local mutations.
4. Assemble an alternate candidate with the inferred bibliography masked so the
   bibliography comes from the XML fallback.
5. Generate deterministic bibliography candidates from typed patch families:
   XML source selection, contributor case, type-local default, date
   granularity, and article-journal suppression.
6. Render citeproc-js bibliography reference strings from the embedded runtime.
7. Render candidates with `citum-engine` in plain text.
8. Keep the incumbent unless a candidate wins on pass count or clears the tie
   margin on summed similarity.

This does not make XML compilation authoritative. It treats XML output as one
candidate in a measured search space. Mutation search is deliberately
non-combinatorial in this implementation: every candidate is a single named
patch with a family, affected section, and affected reference types.

## Acceptance Criteria

- `citum-migrate` can render citeproc bibliography reference strings through
  the embedded runtime.
- The migrator can select inferred, XML, and bounded patch candidates
  empirically.
- Debug output identifies the selected candidate family, affected section,
  affected types, pass delta, and similarity delta.
- The seeded random-100 scorecard remains the headline gate:

```bash
node scripts/report-migrate-sqi.js --corpus random --sample 100 --seed 20260610
```

- Goal state: more than 80 of the 100 sampled styles score at least 90%
  combined strict citation+bibliography fidelity.

## Failure Modes

- Reference-entry mapping can fail for styles whose bibliography output lacks
  enough title, name, or year signal. Such entries are skipped for selection
  rather than guessed.
- Candidate scoring can prefer XML when the current fixture surface is too
  narrow. The tie margin prevents equal-pass noise from flipping the inferred
  default.
- Engine defects remain possible. If both candidates score poorly and the
  oracle diff shows correct template data rendered incorrectly, route the gap
  to `citum-engine`.

## Verification

Targeted:

```bash
cargo test -p citum-migrate measured_citation
node scripts/report-migrate-sqi.js --styles zeitschrift-fur-allgemeinmedizin,brazilian-journal-of-psychiatry
```

Gate before commit for Rust changes:

```bash
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run
```
