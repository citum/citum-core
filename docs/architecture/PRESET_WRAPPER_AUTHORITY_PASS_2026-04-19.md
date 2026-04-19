# Preset Wrapper Authority Pass

**Date:** 2026-04-19
**Related PR:** #552
**Related:** `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md`, `docs/guides/STYLE_WORKFLOW_EXECUTION.md`, `docs/guides/ALIAS_DISCOVERY.md`

## Summary

PR #552 started as a cleanup pass to convert standalone style variants into thin
`preset:` wrappers. During review, it became clear that CSL-derived structure was
often being treated as the authority for how much YAML had to survive. That is
the wrong default.

For dependent journal styles, the correct authority order is:

1. publisher or journal style guide
2. publisher house rules or submission instructions
3. parent-style manual or family reference
4. CSL implementation
5. existing Citum wrapper

The key consequence is that a standalone CSL file can no longer be assumed to be
the minimal truth for a Citum wrapper. It is evidence, not authority.

## Why This Matters

Some journals explicitly describe themselves as following a known parent style
with a few local adjustments. In those cases, the Citum target form should be a
parent preset plus a bounded delta, even if the CSL file duplicates large
templates.

The motivating example raised during PR review was AAG-family guidance. In this
case the proper chain is not a direct jump from AAG to Chicago author-date. The
publisher guidance points to the Taylor & Francis "B" style, and that style is
itself a bounded adaptation of Chicago author-date. In Citum terms, that means
the correct model is layered:

`annals-of-the-association-of-american-geographers`
-> `taylor-and-francis-chicago-author-date`
-> `chicago-author-date-18th`

The repo already encodes that chain in `registry/default.yaml` and
`styles/embedded/taylor-and-francis-chicago-author-date.yaml`. The important
lesson is that future wrapper work should preserve this kind of intermediate
parent when the publisher evidence points to it, rather than collapsing
everything directly onto the oldest common ancestor.

The same pattern likely exists across additional journal families.

## What The Alias TSV Can And Cannot Tell Us

The dated alias-discovery TSV in `scripts/report-data/` is useful for finding
likely parent families. It reports:

- best matching builtin
- overall similarity
- exact citation match rate
- exact bibliography match rate

It does **not** report field-level deltas or prove that a wrapper can be safely
collapsed to a given parent. It is a triage tool, not the authority.

## Findings From The PR #552 Pass

### Confirmed

- The current branch successfully moved the targeted standalone styles onto
  Level 2 `preset:` wrappers.
- Remaining bulk is concentrated in bibliography templates and type-variants,
  not in top-level metadata.
- Several wrappers appear conceptually closer to "parent style plus a few house
  rules" than their surviving YAML size suggests.

### Constraint

The current preset model is still too coarse for some of these styles to become
truly thin wrappers. The main blockers are:

- array replacement semantics for templates and type-variants
- no compact subtractive override for inherited bibliography components
- no fine-grained way to remove inherited component-local settings such as
  `shorten` without replacing the whole affected variant

These constraints encourage carrying larger bibliography blocks than the style
guide alone would justify.

### Working Rule For This PR

For wrapper cleanup work:

- choose the parent from publisher evidence first
- keep only guide-confirmed deltas
- use CSL output and existing fixtures to verify behavior
- do not treat CSL duplication as a reason by itself to retain duplicated Citum YAML

## Follow-Up Implications

Future style-evolution work should assume that "closest CSL parent" and
"canonical style authority" are different questions.

This suggests two follow-ups:

1. workflow: make publisher-guide-first reasoning explicit in style-evolve
   operations
2. infrastructure: add finer-grained preset override mechanics so
   parent-plus-delta wrappers do not need full bibliography array replacement

Until the second item exists, some wrappers will remain bulkier than their
publisher guidance would ideally require.
