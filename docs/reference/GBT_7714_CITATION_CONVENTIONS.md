# GB/T 7714—2025 Citation Conventions

**Version:** 1.0
**Date:** 2026-07-18
**Related:** [DATE_MODEL.md](../specs/DATE_MODEL.md)

Some GB/T 7714—2025 citation forms don't have first-class Citum schema
support: either a documented convention using existing fields, or a
tracked gap.

## Sequel citations (§8.5.1.3)

GB/T 7714—2025 allows a work published serially across multiple issues of
the same journal to append the later parts' `year, volume(issue): pages`
after the first part instead of citing each part separately:

```
2011, 33(2): 20-25; 2011, 33(3): 26-30
```

Citum has no dedicated schema for this — there is no `sequels`/`continued_in`
relation on `SerialComponent`
([structural.rs](../../crates/citum-schema-data/src/reference/types/structural.rs)).
This is deliberate: per reviewer feedback on
[PR #1067](https://github.com/citum/citum-core/pull/1067#issuecomment-5011352331),
the case is rare, Zotero users typically import each part separately, and
the reference LaTeX implementations
([gbt7714-bibtex-style](https://github.com/zepinglee/gbt7714-bibtex-style/blob/5aaf4c5064db6b876e52a6c4a6a59c3f735f6e60/gbt7714-examples.bib#L967-L970),
[biblatex-gb7714-2025](https://github.com/hushidong/biblatex-gb7714-2025/blob/7e43909a3ccee6201009983dec8530c6bad8e83b/example2025.bib#L1209))
don't implement it as structured data either — they cram the extra parts
into an existing field as a literal string.

**Convention:** append the subsequent parts onto `SerialComponent.pages`
(`structural.rs:988`), semicolon-delimited, matching the shape GB/T 7714
itself shows and the reference LaTeX implementations already use:

```yaml
pages: "20-25; 2011, 33(3): 26-30"
```

The engine renders this as an opaque string — no per-part sorting,
localization, or delimiter styling. If structured rendering is ever
needed, see the extension options recorded in the follow-up bean
(csl26-to3s, "Consider extending WorkRelation for GB/T §8.5.1.3 sequel
citations") for the tradeoffs between reusing `WorkRelation::Embedded` and
adding a dedicated structured type.

## Ancient/regnal years (§7.5.4.1)

GB/T 7714—2025 requires a Gregorian publication year to be annotated with
the original calendar's year in parentheses when the source uses a
non-Gregorian calendar, e.g. `1705（康熙四十四年）` (the 44th year of the
Kangxi reign) or `1947（民国三十六年）` (Minguo year 36). See the reviewer's
[explanation](https://github.com/citum/citum-core/pull/1067#issuecomment-5011352331)
and the [Sinosphere era names background](https://en.wikipedia.org/wiki/Regnal_year#Sinosphere_era_names).

This is a genuine gap, not something already covered by Citum's existing
"ancient date" support. [EDTF_HISTORICAL_ERA_RENDERING.md](../specs/EDTF_HISTORICAL_ERA_RENDERING.md)
and [EDTF_ERA_LABEL_PROFILES.md](../specs/EDTF_ERA_LABEL_PROFILES.md) cover
only a fixed BC/AD/BCE/CE suffix on *negative* EDTF years (astronomical→
historical `1 - year` conversion, implemented in `format_display_year`,
[date.rs](../../crates/citum-engine/src/values/date.rs)) — explicitly
scoped to exclude "automatic AD/CE rendering for positive years." Chinese
regnal-year annotation needs a different, larger capability: era-name/dynasty→
Gregorian-span lookup tables, ordinal-year formatting, and a new
parenthetical dual-year render shape. Tracked in a follow-up bean
(csl26-0kqf, "Support GB/T 7714 §7.5.4.1 regnal/era-year annotations");
not attempted in PR #1067.

**Input capture, decoupled from calendar computation.** The data-model
question this raised — where does the source-calendar wording come from on
input? — is answered by the [Date Annotations
specification](../specs/CALENDAR_DATE_ANNOTATIONS.md): every EDTF date field
accepts an optional opaque `note` sub-field (`issued: { value, note }`) that a
style can wrap and render, e.g. `note-wrap: parentheses` in a bibliography's
`dates` options. `note` is verbatim, uninterpreted display text — it does not
identify the calendar system, convert the year, or resolve a dynasty/era
lookup, so it captures exactly the `1705（康熙四十四年）` shape above without
depending on the (still-unimplemented) computed regnal-year capability
`csl26-0kqf` tracks.
