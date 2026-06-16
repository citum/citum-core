---
# csl26-auh3
title: 'migrate: unhandled CSL variables — event, collection-title, translator, original-date'
status: todo
type: feature
priority: normal
created_at: 2026-06-16T15:49:01Z
updated_at: 2026-06-16T15:49:01Z
---

After coverage-gap key-mapping fixes (csl26-t56t), these variables remain as genuine converter gaps across the independent-style corpus:

Rank / Feature / Affected styles (approx):
- var:event (1249): conference/event name. Not in SimpleVariable or TitleType. May need SimpleVariable::Event or a dedicated EventTitle component.
- var:collection-title (1669): series title. Not a TitleType variant — no TitleType::Series. May need a new TitleType or a SimpleVariable passthrough.
- names:translator (715): translator contributor role. Check if ContributorRole has a Translator variant; if not, add it and wire into the contributor compiler.
- date:original-date (470): original publication date. Check if DateVariable has OriginalDate; if not, add it.
- var:year-suffix (428): author-date disambiguation suffix. This is processor-generated, not a template variable — converter correctly omits it; document that it's handled by engine disambiguation, not by template.
- var:section (753), var:authority (704): verify these are in SimpleVariable (Section and Authority exist) and trace why migrate drops them.

Approach: for each item, trace the csl_legacy node compiler path (crates/citum-migrate/src/template_compiler/node_compiler.rs) to see whether the variable is handled, silently dropped, or needs a schema addition.

Priority sequence: collection-title (1669) > translator (715) > event (1249) > original-date (470).
