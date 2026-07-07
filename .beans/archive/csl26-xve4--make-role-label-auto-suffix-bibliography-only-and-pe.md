---
# csl26-xve4
title: Make role-label auto-suffix bibliography-only and per-style
status: completed
type: task
priority: normal
tags:
    - contributors
created_at: 2026-07-06T14:02:46Z
updated_at: 2026-07-07T15:50:33Z
parent: csl26-8m2p
---

div-012 (docs/adjudication/DIVERGENCE_REGISTER.md) documents that resolve_role_labels (crates/citum-engine/src/values/contributor/labels.rs) applies one hardcoded Long-form short-suffix default (APA's "(ed.)"-shaped treatment) to all seven roles (editor, chair, translator, interviewer, director, illustrator, composer) uniformly, regardless of which style is active.

Perplexity-researched style-guide conventions (recorded in the PR #1017 conversation, corrected after an initial answer conflated bibliography and in-text conventions) show two separate problems, not one:

1. Role labels are a bibliography/reference-list-only convention in every style examined -- no style (APA, MLA, Chicago, Vancouver/NLM) carries a role label into the in-text citation name string, even when the same contributor occupies the author slot there too. APA appends "(Ed.)"/"(Eds.)" only in the reference entry and cites by surname+year alone in text; MLA requires a word-form label ("editor", "director", ...) only in the Works Cited "author" element, not in the in-text parenthetical. `resolve_role_labels` (crates/citum-engine/src/values/contributor/labels.rs) never reads `RenderContext`, so today's hardcoded default fires identically in a citation template as in a bibliography template -- this is a real correctness gap, not just a per-style preference question.
2. Even within bibliography context, the specific default (APA's editor-shaped short-suffix) is not a universal per-role rule: APA labels only editor; MLA labels a broader set (editor, translator, director, performer, artist) but with word-form labels, not APA's abbreviation; Chicago avoids parenthetical author-slot labels almost entirely, preferring "edited by ..." after the title; Vancouver/NLM is inconsistent.

Only `editor` has real per-style backing (APA) even in bibliography context; the other six roles (chair, translator, interviewer, director, illustrator, composer) share the same treatment with no confirmed style-guide justification for any of them. The current default is kept only because real embedded styles (e.g. elsevier-with-titles-core.yaml) depend on it with zero configuration -- it's legacy behavior, not a deliberate cross-style design choice.

## Scope
Two parts:
1. Make `resolve_role_labels` context-aware: the hardcoded Long-form auto-label default should only apply in bibliography context, never in citation context, matching every examined style's actual convention. Check whether any embedded style currently relies on the auto-label firing in a citation template before changing this (a pre-flight sweep, same approach used for the original div-012 finding).
2. Design a schema-level mechanism (e.g. default presets keyed by processing family, or an explicit per-style declaration) so a style's bibliography-context role-label defaults can match its own actual convention (APA: editor only, abbreviated; MLA: broader set, word-form; Chicago: none/prefer post-title phrasing) instead of one engine-wide default serving every style.

Must not change output for any style currently passing oracle fidelity without that style explicitly opting in to the new configurability -- i.e. preserve today's behavior as the default for styles that don't specify anything, the same way div-011's title-quote gate did.

## References
- div-012 in docs/adjudication/DIVERGENCE_REGISTER.md
- crates/citum-engine/src/values/contributor/labels.rs (resolve_role_labels)
- crates/citum-schema-style/src/options/contributors.rs (RoleOptions, RoleLabelPreset)
- Audit finding 16b, docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md
- Bean csl26-mc0c (closed the audit finding without this deeper fix; PR #1017)

## Implementation Notes (2026-07-07)

- Baseline vs after `report-core.js` diff: only chicago-author-date-18th and taylor-and-francis-chicago-author-date changed, both IMPROVED (+1 bibliography pass each; the removed " (interviewer)" suffix moved output toward the citeproc-js oracle). No embedded style regressed, so none needed compensating YAML config.
- User decision: no `legacy` compatibility variant. Engine default is `none`; bundles `apa` (editor-only short-suffix) and `mla` (word-form long-suffix for its documented role set) are opt-in, bibliography-context only.
- Spec: docs/specs/ROLE_LABEL_DEFAULTS.md; div-012 marked resolved.

## Summary of Changes

- Removed the engine-hardcoded 7-role Long-form auto-suffix from resolve_role_labels (crates/citum-engine/src/values/contributor/labels.rs). No legacy variant kept: the engine default is now no automatic role label.
- Added RoleLabelDefaults bundles (none/apa/mla) + RoleOptions.defaults field (crates/citum-schema-style/src/options/contributors.rs), applied only in bibliography context and never for verb/verb-short forms; per-role presets and explicit labels still win and remain context-independent.
- Fidelity: before/after report-core diff shows zero regressions; chicago-author-date-18th and taylor-and-francis-chicago-author-date each gained +1 bibliography pass (the old " (interviewer)" suffix diverged from the citeproc-js oracle). No embedded style needed compensating YAML.
- Tests: new schema round-trip/bundle tests, bibliography defaults-bundle tests, citation-context suppression test; updated tests that encoded the old implicit default.
- Docs: spec docs/specs/ROLE_LABEL_DEFAULTS.md (Active); div-012 marked resolved in DIVERGENCE_REGISTER.md; schemas regenerated.
