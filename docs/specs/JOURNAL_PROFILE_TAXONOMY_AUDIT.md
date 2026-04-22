# Journal-Profile Taxonomy Audit

**Status:** Draft
**Date:** 2026-04-22
**Related:** `STYLE_TAXONOMY.md`, `UNIFIED_SCOPED_OPTIONS.md`, `../architecture/2026-04-22_JOURNAL_PROFILE_CANDIDATE_AUDIT.md`, commit `9e13a17b`

## Summary

This audit revisits the `9e13a17b` "Profile Candidates" backlog after the
project introduced config-only public profile wrappers.

The result is a taxonomy correction plus a bounded reduction pass:

- `profile` remains a semantic class **and** an implementation contract
- `journal` becomes semantic only
- journal descendants may now be represented as `alias`, `config-wrapper`, or
  `structural-wrapper`

The audited shortlist produced:

- `3` journal-profile promotions
- `0` new aliases
- `0` converted journal-structural descendants
- `5` false positives, dropped parent links, or temporary legacy holds

## Findings

### 1. The old tier model mixed two different questions

The prior table answered both:

- what relationship a style has to a parent
- how that relationship is currently encoded

That works for `base` and `profile`, but it breaks down for journals because the
repo already contains all three of these:

- pure registry aliases
- thin journal wrappers
- journal descendants with local templates

### 2. `9e13a17b` was useful as family triage, but not precise enough for promotion

The shortlist still surfaced several real family signals:

- `pharmacoepidemiology-and-drug-safety`
- `disability-and-rehabilitation`
- `memorias-do-instituto-oswaldo-cruz`
- `hawaii-international-conference-on-system-sciences-proceedings`
- `cell-numeric`

The second-pass reduction changed that result materially:

- `pharmacoepidemiology-and-drug-safety`
- `disability-and-rehabilitation`
- `cell-numeric`

all reduce to config-only wrappers once copied CSL structure is treated as
suspect and removed unless a guide-backed delta still requires it.

One other did not justify keeping an inherited parent at all:

- `memorias-do-instituto-oswaldo-cruz`

`hawaii-international-conference-on-system-sciences-proceedings` remains a
special case: current guide evidence contradicts the inferred IEEE family, but
the legacy CSL-compatible Citum style still temporarily extends IEEE because
that inherited surface materially preserves fidelity.

### 3. Correct parent selection must prefer the nearest current Citum hub

The audit confirmed two parent corrections that matter immediately:

- `pharmacoepidemiology-and-drug-safety` should be evaluated against
  `american-medical-association`, not `elsevier-with-titles`
- `hawaii-international-conference-on-system-sciences-proceedings` should be
  evaluated against `ieee`, not `taylor-and-francis-national-library-of-medicine`

## Analyzer Contract

`citum-analyze --identify-profiles` now acts as an audit-oriented triage report
 for the current shortlist instead of printing raw semantic-similarity hits.

The new report:

- normalizes candidate IDs before lookup
- joins CSL metadata, registry knowledge, and the dated alias TSV
- reports corrected parent mappings separately from the originally proposed
  parent
- emits structured JSON via `--json`

This is intentionally conservative. The tool should surface evidence and
disposition, not silently promote styles.

## Recommendation

Adopt the two-axis taxonomy from `STYLE_TAXONOMY.md`:

- semantic class: `base`, `profile`, `journal`, `independent`
- implementation form: `alias`, `config-wrapper`, `structural-wrapper`,
  `standalone`

Keep the runtime `kind` enum unchanged in this pass. If the project later needs
machine-readable implementation-form metadata, add a new field instead of
redefining `kind`.

## Defaults Chosen In This Pass

- three journal candidates are promoted to config-only wrapper
- no journal candidate is added as a registry alias
- one journal candidate drops its parent link and remains standalone
- one journal candidate remains a documented temporary legacy hold rather than a
  taxonomy-backed parent mapping
- false positives and unsupported parent links are kept explicit rather than
  silently dropped from history
