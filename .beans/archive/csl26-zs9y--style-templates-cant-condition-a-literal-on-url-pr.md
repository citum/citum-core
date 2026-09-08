---
# csl26-zs9y
title: Style templates can't condition a literal on URL presence
status: completed
type: task
priority: normal
tags:
    - schema
    - fidelity
    - style
created_at: 2026-09-05T21:24:36Z
updated_at: 2026-09-08T21:23:11Z
parent: csl26-ccdt
---

NLM-family styles (T&F-NLM, and IEEE has a similar case) need a literal
"[Internet]" marker appended to the TITLE component, but only for
entries that have a URL (i.e. are online-only sources) -- and
separately "Available from: " as the URL's own prefix, and "[cited
DATE]" attached to the accessed date. The latter two are already free
(a `variable: url` or `date: accessed` component auto-suppresses when
its own field is empty), but the title-suffix marker has no field of
its own to piggyback on.

The only field-presence gate in the schema is `TemplateGroup.render_when:
TemplateGroupCondition` (crates/citum-schema-style/src/template.rs:1790-1816),
and it only exists on `group` components -- not on `title`,
`variable`, or any other leaf component. Its `TemplateConditionField`
enum (same file, ~1822-1870) also has no `Url`/`Doi`-presence variant
usable for this (it has `Doi` but not `Url`).

Found tuning csl26-on47 (T&F-NLM: ~8 residual rows across webpage,
dataset, interview, map, hearing, software types all need this same
"[Internet]. ... Available from: URL." pattern; IEEE has an analogous
"[Online]. Available: URL" pattern for similar types). Both are
blocked on the same missing primitive.

Per docs/guides (schema changes need a docs-first spec before
implementation): this needs a spec in docs/specs/ proposing either (a)
a `Url` variant added to `TemplateConditionField` plus wiring
`render_when` (or an equivalent) onto `TemplateTitle` and other leaf
components, or (b) a narrower purpose-built "online-source marker"
component. Scope: engine (`crates/citum-engine`) + schema
(`crates/citum-schema-style`), not style YAML.

## Resolution direction (2026-09-06)

Root cause reclassified: this is not a render-when gap (option (a) above), it's option (b) -- a narrower purpose-built option. Spec drafted at `docs/specs/MEDIUM_DESIGNATOR.md` (Draft): a `BibliographyConfig.online_access` option bundling the title-suffix marker, access-phrase, and cited-date-bracket, gated by URL presence and an excluded-type list. Decided alongside the broader `render-when` disposition question -- see `docs/architecture/audits/2026-09-06_RENDER_WHEN_DISPOSITION.md`.

Confirmed against the shipped .csl: `taylor-and-francis-national-library-of-medicine.csl:133-150` (title macro) and `springer-vancouver-brackets.csl:113-120` (accessed-date macro). Also affects `taylor-and-francis-council-of-science-editors-author-date`. IEEE's analogous "[Online]. Available: URL" pattern (noted above) is a plausible second consumer once this option exists, not part of this spec's acceptance criteria.

Next: spec review, then implementation in a stacked PR.

## Summary of Changes

Resolved via docs/specs/MEDIUM_DESIGNATOR.md's online-access bibliography option (csl26-r4dn), not the render_when/TemplateConditionField::Url options this bean originally proposed -- root cause reclassified 2026-09-06 (see the bean's own Resolution direction note) to a narrower purpose-built option instead. The [Internet] marker, cited-date bracket, and title-suffix anchor-selection gap are all implemented and wired for taylor-and-francis-national-library-of-medicine, springer-vancouver-brackets, and taylor-and-francis-council-of-science-editors-author-date. IEEE's analogous [Online] pattern remains a plausible follow-up, not covered here.
