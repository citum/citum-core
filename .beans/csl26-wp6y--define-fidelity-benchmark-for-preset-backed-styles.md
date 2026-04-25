---
# csl26-wp6y
title: Define policy and guardrails for preset-backed styles with local overrides
status: todo
type: task
priority: normal
tags:
    - policy
    - style
created_at: 2026-03-18T19:57:26Z
updated_at: 2026-04-25T20:20:07Z
---

We need a follow-up policy decision for how preset-backed styles should be authored, interpreted, and benchmarked once a style file references a style preset and then overrides some fields locally.

## Context
Discovered while completing csl26-fsjy. The JS fidelity/reporting tooling now distinguishes between raw authored YAML (wrapper identity / baseline selection) and resolved effective style data (behavior, note/bibliography classification, quality metrics). That fixed the immediate CI breakage, but it does not answer the deeper benchmark question for partially overridden preset-backed styles.

The new `citum-analyze --quantify-savings` report also makes the stakes more
concrete: on the current CSL snapshot, presets plus locale overrides appear to
avoid between 7,998 and 10,138 one-by-one conversions, depending on how broadly
we count template-linked wrapper opportunities. That makes it more important to
define when preset-backed wrappers remain a reuse layer versus when they should
be treated as effectively independent styles.

The implementation is good enough to ship, but it leaves some policy questions intentionally open:
- authored identity vs resolved effective behavior
- fidelity comparator / oracle identity
- the threshold at which a thin wrapper becomes effectively its own style
- authoring ergonomics for replace-only arrays and large local override layers
- whether preset-backed wrappers need linting or other complexity guardrails

## Questions
When a style uses `preset:` and also adds local overrides:
- The wrapper style filename / declared style identity?
- The preset base style's legacy CSL source?
- A separate benchmark implied by the effective resolved style?
- Some threshold/policy based on how large the local delta is?

At what point should a preset-backed style stop being treated as a compact wrapper and instead be considered an independent style for maintenance and verification purposes?

What guidance should authors get for when not to use a preset, especially when arrays and templates need full replacement?

## Why this matters
Different answers affect oracle routing, regression baselines, SQI/core-quality reporting, author comprehension, and how we interpret fidelity claims for derived styles. The immediate csl26-fsjy fix preserved wrapper-style identity for now, but that is a pragmatic default rather than a settled project rule.

## Likely outputs
- Decide the fidelity authority/comparator rule for preset-backed override styles
- Define guidance for authored identity vs resolved behavior in reports and tooling
- Define authoring guidance for when presets are appropriate vs when a full style is clearer
- Consider linting or complexity thresholds for large preset-backed override layers
- Document the rule set (spec or policy)
- Update JS/Rust verification tooling if needed

Related: csl26-fsjy, docs/specs/STYLE_PRESET_ARCHITECTURE.md
