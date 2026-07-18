---
# csl26-838l
title: Per-item term localization (autolang)
status: todo
type: feature
priority: normal
tags:
    - multilingual
    - locale
created_at: 2026-07-18T20:32:33Z
updated_at: 2026-07-18T20:38:10Z
parent: csl26-0ugp
---

Rendering an item's terms in the item's language (German "hrsg. von" for a German source in an English-locale Chicago style) currently requires a citation.locales/bibliography.locales branch that swaps the whole template. Add an opt-in that switches locale-sensitive term/message/date-pattern lookups to the effective item language without changing template structure — the biblatex autolang analogue. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(g).

Spec: docs/specs/PER_ITEM_TERM_LOCALE.md (Draft). Opt-in options.multilingual.term-locale: style | item at the three MultilingualConfig scopes; item mode switches roles/terms/messages/date patterns to the effective item language's locale with exact-tag -> primary-language -> style-locale fallback. Typography (grammar-options) stays style-locale in v1; locale-scoped layout branches take precedence. Usefulness bounded by embedded locale coverage (csl26-tfi8, csl26-itri).
