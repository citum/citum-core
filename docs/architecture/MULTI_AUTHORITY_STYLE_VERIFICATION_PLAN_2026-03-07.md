# Multi-Authority Style Verification Plan

Date: 2026-03-07
Status: Proposed
Bean: `csl26-ccqg`
Bean file: `.beans/csl26-ccqg--define-multi-authority-style-verification-model.md`

## Summary

Citum has outgrown the older "oracle" framing. The project now needs a more
precise model that distinguishes between:

- authority sources that define correct output
- regression baselines that detect unintended change
- comparators that run the checks
- adjudication records that explain why one source wins over another

This plan defines that model, applies it to the current CSL and biblatex
portfolio, and makes documentation updates part of the same scope.

The machine-readable policy for this work should live in
`scripts/report-data/verification-policy.yaml`, with optional human-readable
adjudication notes in `docs/adjudication/`.

The most important new capability in this plan is the fixture-sufficiency
layer, so authority decisions are not made from default reports that omit the
reference types and scenarios where differences actually appear.

## Problem

The current tooling still assumes that citeproc-js is the universal source of
truth. That is no longer accurate.

Three different kinds of verification sources already exist in the repo:

1. citeproc-js snapshots and live comparisons for CSL-derived styles
2. biblatex-generated snapshots for chemistry and compound-numeric styles
3. Citum snapshot text files for styles that were authored directly in Citum

The last category is especially important to name correctly. A Citum-authored
snapshot is not an external authority source. It is a locked regression
baseline: "this previously approved Citum output should not change by accident."

Keeping all of these under the vague label "oracle" now hides the real policy
question: which source is authoritative for which style, and what should happen
when multiple sources disagree?

The current default fixture set is also too limited to support all authority
decisions safely. Some meaningful citeproc-js vs biblatex differences are
likely to appear only in reference types or scenarios that are underrepresented
or absent from the default reports. Without an explicit sufficiency policy, the
project can gain false confidence from a clean report that never exercised the
relevant cases.

## Terminology Decisions

Use the following vocabulary in docs, plans, reports, and future tooling:

- **Authority source**: the source that defines the expected output for a style
  or scope
- **Secondary reference source**: an additional source used for comparison,
  confidence, or conflict discovery, but not the release gate
- **Regression baseline**: a stored approved output snapshot used to detect
  unintended change when no better authority source exists
- **Adjudication record**: a documented decision that names the winning source
  for a style, scope, or component and explains why
- **Comparator**: the script or harness that runs a specific comparison

Retire "oracle" as the preferred architectural term. It can remain in legacy
script names for continuity, but new docs should use the terms above.

## Policy Decisions

### 1. Precedence hierarchy

For the first implementation wave, the precedence policy should distinguish
between the preferred authority path and the default fallback path.

Preferred authority path:

1. documentary evidence, when a public style guide or journal instructions
   provide usable examples that can be curated into expected outputs
2. citeproc-js / CSL
3. biblatex
4. Citum regression baseline

Default fallback path for most existing CSL-derived styles:

1. citeproc-js / CSL
2. biblatex
3. Citum regression baseline

This is a starting policy, not a permanent claim that citeproc-js is always
more correct. The biblatex-vs-citeproc comparison work in this plan exists
specifically to validate or revise the default where evidence shows otherwise.

Documentary evidence remains a first-class authority source when it is
available in a usable form. The plan should simply avoid assuming that
authoritative manuals for large styles such as APA or Chicago are always
publicly accessible or easy to encode into machine-checked expectations on day
one.

### 2. Granularity

The verification model is style-level first, with documented overrides only
where needed.

Each style should declare:

- one primary authority source
- zero or more secondary reference sources
- supported scopes such as `citation` and `bibliography`

This policy should be external metadata. It should not be added to style YAML
under `styles/`, and it should not be stored under `tests/snapshots/`, which is
already the home for generated comparison artifacts.

Overrides are allowed only when explicitly documented for:

- `citation`
- `bibliography`
- a named component such as pages, DOI, or contributor formatting

### 3. CI behavior

CI should gate only the declared primary authority source.

Conflicts with secondary reference sources should:

- appear in reports
- be tracked as adjudication work
- not fail CI until the style is explicitly reclassified

### 4. Citum snapshots

Citum snapshot files should be treated as regression baselines, not as external
authority sources.

They are appropriate only when:

- the style has no credible external renderer
- the style is intentionally native to Citum
- no curated documentary evidence exists yet

If a better authority source later becomes available, the regression baseline
can remain as a local guardrail, but it should no longer be described as the
source of truth.

### 5. Fixture sufficiency

Authority decisions must be gated by fixture sufficiency, not just by whatever
the default compatibility report happens to cover.

Separate these concerns:

- the default release-report fixture, which should stay stable and fast
- the comparison corpus needed to justify an authority-source promotion or
  adjudication

No style should be promoted from `citeproc-js` to `biblatex`, or declared
"equivalent across sources," unless the comparison corpus covers the reference
types and scenarios that style family is known to stress.

## Implementation Plan

### 1. Add a verification policy registry

Create a machine-readable registry that is the single source of truth for style
verification policy. Each style entry should declare:

- style slug
- primary authority source kind and identifier
- secondary reference sources
- supported scopes
- evidence links or snapshot identifiers
- optional scope or component overrides
- adjudication status

Place this registry at `scripts/report-data/verification-policy.yaml`.

Use a defaults block so unlisted styles inherit the current CSL-era baseline:

- primary authority: `citeproc-js`
- secondary sources: none
- scopes: `citation`, `bibliography`

This registry should be consumed by reporting, regression checks, and
style-author workflows.

Detailed conflict write-ups should live separately under
`docs/adjudication/<style-slug>.md` and be created only when a style needs a
human rationale that is longer than a compact registry entry.

### 1a. Add fixture-sufficiency policy

Create a companion policy file at `scripts/report-data/fixture-sufficiency.yaml`
that declares, per style or style family:

- the required reference types for authority comparison
- the required scenario classes
- whether the default report fixture is sufficient
- which family comparison fixture set must be consulted before promotion

This file should let tooling answer:

- is this style under-tested for authority comparison?
- are the current defaults enough to justify a source switch?
- which extra fixture family must be run?

### 2. Refactor comparators around authority-source types

Unify the current comparison scripts behind a shared resolver that selects the
right comparator from the registry instead of hard-coded heuristics.

The active source types for the first iteration should be:

- documentary snapshot comparator
- biblatex snapshot comparator
- citeproc-js comparator
- Citum regression baseline comparator

The resolver should also understand fixture tiers:

- core release-report fixtures
- family comparison fixtures used for authority analysis
- optional targeted adjudication fixtures for a single style conflict

### 3. Apply the model to the current style portfolio

#### Switch to biblatex-primary now

These styles already have biblatex evidence and should stop using Citum
self-snapshots as the release authority:

- `numeric-comp`
- `chem-acs`
- `angewandte-chemie`
- `chem-rsc`
- `chem-biochem`

#### Create missing styles with biblatex-primary policy

Add new Citum styles for:

- `science`
- `physical-review`

#### Add biblatex as a secondary reference source first

For established CSL-backed styles with strong existing citeproc-js parity, keep
the current release authority in the first wave and use biblatex to surface
conflicts that need adjudication:

- `apa-7th`
- `ieee`
- `nature`
- `modern-language-association`
- `chicago-author-date`
- `chicago-notes`
- `american-physics-society`
- `american-institute-of-physics`
- `american-chemical-society`
- `royal-society-of-chemistry`

#### Treat family styles as references, not end-user slugs

Use `numeric`, `numeric-comp`, `authoryear`, and `authoryear-comp` as family
reference points for adjudication and design. Do not create new end-user style
slugs unless there is a clear product need.

#### Add family comparison fixtures

Add targeted comparison fixtures for the style families most likely to expose
source differences that the core report misses, starting with:

- chemistry / compound numeric
- physics numeric
- author-date
- note / humanities
- legal, using adapted CSL-M fixtures in the first wave

These fixtures should cover the reference types and examples that are likely to
surface the corner cases where biblatex or documentary evidence is more
precise than citeproc-js.

The default compatibility report may remain smaller, but authority promotions
must consult the relevant family fixture when the sufficiency policy requires
it.

When a journal or publisher style guide includes public examples, the relevant
family fixture should capture those examples explicitly so a documentary-primary
path is available instead of forcing all style creation through citeproc-js or
biblatex comparisons.

### 4. Update documentation as part of the same work

The following documentation areas must be updated in the same implementation
wave:

- architecture docs that define testing and source-of-truth policy
- style-author and migration workflow guides
- compatibility and status report language
- any docs that still say citeproc-js is the universal oracle
- examples and dashboard copy that expose verification status to users
- references to registry location and adjudication note location
- fixture-sufficiency policy and family comparison fixture workflow

Documentation updates are part of the definition of done, not a later cleanup
task.

## Acceptance Criteria

This plan is complete when:

- the terminology shift from "oracle" to explicit verification terms is written
  into architecture and workflow docs
- the verification registry exists and is used by reporting and checks
- the registry lives in `scripts/report-data/verification-policy.yaml`
- fixture sufficiency is recorded in `scripts/report-data/fixture-sufficiency.yaml`
- biblatex-primary chemistry and compound styles are no longer graded against
  Citum self-snapshots as their release authority
- `science` and `physical-review` are on the rollout path as biblatex-primary
  styles
- legal comparison coverage is included through adapted CSL-M fixture intake
- mixed-source conflicts are visible in reports without breaking CI by default
- the remaining use of "oracle" is limited to legacy script names or historical
  context
- longer conflict justifications live under `docs/adjudication/` when needed
- authority-source promotions are blocked when the relevant family fixture
  coverage is missing

## Follow-up Constraint

Biblatex comparison should be treated as bibliography-first in the first
iteration. Citation arbitration for biblatex-backed author-date and note styles
needs dedicated fixture design and should be handled as a follow-up once the
authority-source registry, sufficiency policy, and terminology are in place.
