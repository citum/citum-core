---
# csl26-fqz2
title: Add per-component override for delimiter-precedes-last at N=2
status: todo
type: task
priority: normal
tags:
    - contributors
created_at: 2026-07-06T14:03:06Z
updated_at: 2026-07-06T14:03:13Z
parent: csl26-8m2p
---

div-013 (docs/adjudication/DIVERGENCE_REGISTER.md) documents that join_names_with_conjunction (crates/citum-engine/src/values/contributor/names.rs) unconditionally suppresses the delimiter before the conjunction for exactly two names in citation context, and in a given-first bibliography name list, regardless of the declared delimiter-precedes-last value.

This suppression's shape matches a real, documented APA convention (Perplexity-researched, recorded in the PR #1017 conversation): inverted (family-first) reference-list author lines take the comma even at two names ("Smith, J., & Jones, K."), while non-inverted/narrative name lists and in-text citations reserve the comma for three-plus. But Citum applies this as an engine-wide default regardless of which style is active, and it's APA-specific, not confirmed universal (MLA's rule is similar in shape but doesn't make the same inverted/non-inverted split; Chicago/Vancouver document serial-comma rules style-wide with no stated dependence on name inversion).

The real gap: a style other than APA that sets delimiter-precedes-last: always and genuinely wants the comma at N=2 for a given-first or citation-context list has no way to ask for it today -- there is no per-component or per-style override, only the one global ContributorConfig.delimiter_precedes_last field.

## Scope
Add a way for a style to opt out of the N=2 suppression (or opt into literal `always` semantics) -- e.g. a per-component override on TemplateContributor, or a context-aware variant/flag alongside DelimiterPrecedesLast -- mirroring how div-011's title-quote gate was added (new schema field, default preserves today's behavior exactly, no embedded style needs to change). Must not regress apa-7th's oracle-verified citation (20/20) or bibliography fidelity.

## References
- div-013 in docs/adjudication/DIVERGENCE_REGISTER.md
- crates/citum-engine/src/values/contributor/names.rs (join_names_with_conjunction)
- crates/citum-schema-style/src/options/contributors.rs (DelimiterPrecedesLast)
- Audit finding 9, docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md
- Bean csl26-mc0c (closed the audit finding without this deeper fix; PR #1017)
