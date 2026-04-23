# Style-Evolve Profile Elsevier Wave

**Date:** 2026-04-23
**Bean:** `csl26-u1tq`
**Related:** `docs/specs/STYLE_TAXONOMY.md`, `docs/specs/UNIFIED_SCOPED_OPTIONS.md`, `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md`

## Summary

This pass bundled a bounded Elsevier profile-family verification wave with the
profile-aware `style-evolve` workflow update.

The intended question was simple:

- do dedicated Elsevier family roots exist
- do the public Elsevier profile handles already resolve as true thin wrappers
- does the shared workflow now describe how to make and verify that judgment

The answer is yes on all three counts.

## Current Elsevier Family Shape

The repo already contains dedicated hidden family roots and public thin wrappers
for the two target Elsevier profiles:

- `styles/embedded/elsevier-harvard-core.yaml`
- `styles/embedded/elsevier-harvard.yaml`
- `styles/embedded/elsevier-vancouver-core.yaml`
- `styles/embedded/elsevier-vancouver.yaml`

Current wrapper shape:

- both public wrappers use `extends:`
- both wrappers limit local deltas to metadata and scoped options
- neither wrapper carries local templates
- neither wrapper carries local `type-variants`
- `elsevier-harvard` changes bibliography date placement through
  `bibliography.options.date-position`
- `elsevier-vancouver` changes citation label wrapping and bibliography label
  mode through scoped options only

That means no YAML reduction work was required in this pass. The wave outcome is
verification plus workflow alignment, not synthetic churn.

## Evidence

Authority basis:

1. current Elsevier guide links embedded in the style metadata
2. current family-root and wrapper YAML structure
3. style-oracle and style-scoped report verification on the public handles
4. shared-fixture parent-vs-wrapper diffs

Operational classification:

- `elsevier-harvard`: `profile + config-wrapper`
- `elsevier-vancouver`: `profile + config-wrapper`

Verification summary:

- `node scripts/oracle-yaml.js styles/embedded/elsevier-harvard.yaml styles-legacy/elsevier-harvard.csl --json`
  returned `18/18` citations and `34/34` bibliography
- `node scripts/oracle-yaml.js styles/embedded/elsevier-vancouver.yaml styles-legacy/elsevier-vancouver.csl --json`
  returned `18/18` citations and `34/34` bibliography
- `node scripts/report-core.js --style elsevier-harvard` reported quality score
  `0.907`
- `node scripts/report-core.js --style elsevier-vancouver` reported quality
  score `0.847`
- parent-vs-wrapper rendering on the shared fixtures showed that the wrapper
  deltas are real but bounded:
  `elsevier-harvard` changes bibliography layout relative to the family root,
  while `elsevier-vancouver` changes label rendering relative to the family
  root

## Decision

Keep the existing Elsevier family-root and wrapper YAML unchanged.

The real repo gap was not missing Elsevier roots. It was that the shared
workflow and host wrappers did not explicitly tell agents how to:

- classify semantic class vs implementation form
- preserve the config-wrapper contract for profiles
- accept structural journal descendants when the evidence or infrastructure
  requires them
- stop forcing thin reductions when current merge mechanics block them

This pass addresses that workflow gap directly.

## Follow-Up Boundary

This pass does not widen into schema or runtime work.

If a future family re-check shows that an Elsevier public handle needs more than
scoped options and metadata, the correct action is to record the infrastructure
constraint or author a different family base. It is not to silently treat copied
structural YAML as authoritative.
