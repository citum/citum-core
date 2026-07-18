---
# csl26-838l
title: Per-item term localization (autolang)
status: todo
type: feature
tags:
    - multilingual
    - locale
created_at: 2026-07-18T20:32:33Z
updated_at: 2026-07-18T20:32:33Z
parent: csl26-0ugp
---

Rendering an item's terms in the item's language (German "hrsg. von" for a German source in an English-locale Chicago style) currently requires a citation.locales/bibliography.locales branch that swaps the whole template. Add an opt-in that switches locale-sensitive term/message/date-pattern lookups to the effective item language without changing template structure — the biblatex autolang analogue. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(g).
