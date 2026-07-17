---
# csl26-ra71
title: 'GB/T numeric wave-3: remaining structural gaps after zmod+49sj'
status: todo
type: task
priority: normal
tags:
    - style
    - fidelity
    - multilingual
    - dates
created_at: 2026-07-17T10:22:18Z
updated_at: 2026-07-17T11:01:24Z
---

Remaining ~9 genuine gaps in gb-t-7714-2025-numeric after csl26-zmod (structural long tail) and csl26-49sj (conditional number labels) both landed, bringing the upstream corpus to 190/203. Not counting the 4 ordinal-form entries (tracked separately, see csl26-g49a).

All style fixes below go in the shared hidden base, `crates/citum-schema-style/embedded/styles/gb-t-7714-2025-base.yaml`, so note/author-date inherit them too. Verify each against:
`node scripts/oracle.js tests/fixtures/csl-m/gb-t-7714-2025-numeric.csl --refs-fixture tests/fixtures/test-items-library/gb-t-7714-2025.json --scope bibliography --json`

1. **Container-title's own volume-in-chapter** (gbt7714.8.3.2:2, 8.3.2:3). The `book,thesis,map:` variant's own title already gets a `group: [title, number:volume when-numeric:short]` circumfix (added this wave). The `chapter,entry-dictionary,entry-encyclopedia:` variant's *container*-title component needs the identical group, applied to `title: container-title` instead of `title: primary`, in `gb-t-7714-2025-base.yaml`.

2. **Volume-title note-field append** (gbt7714.7.2.3:5, `type: book`). Raw fixture: `note: "volume-title: 科学思想史"`. `crates/csl-legacy/src/csl_json.rs`'s note-hack parser already recognizes `volume-title` as a string variable and stores it to `extra`; `crates/citum-schema-data/src/reference/conversion/scholarly.rs:82,203` already reads it via `legacy_extra_str(&legacy, "volume-title")` — but only inside whichever function handles media-type conversion, as a *title fallback*, not for `book` type. `book`-type items route through a different conversion function (not `from_document_ref` — that's map/figure/graphic/periodical/collection/document); need to trace which one handles plain `book` (likely in `crates/citum-schema-data/src/reference/conversion/mod.rs`) and wire `volume-title` into a new renderable field, then add a template component after `number: volume` in the title group, joined by a space.

3. **Circa/approximate issued date** (gbt7714.7.5.4.3:1). Oracle wants `c1988` (compact, no space, no `ca.`); citum drops the date entirely. `options.dates.approximation-marker: 'ca. '` is already set in the style; the engine's approximation handling is in `crates/citum-engine/src/values/date.rs`. Check the raw fixture's `issued` field for this item first — need to confirm what actually marks it as circa (EDTF uncertainty flag vs. a note-field override) before touching the date component, since the marker isn't reaching the renderer at all right now (not just wrong formatting).

4. **Periodical open-range residual** (gbt7714.8.4.2:2, `type: book` with `note: "type: periodical\nissued: 1957/1990\ntex.entrytype: periodical"`). Raw `issued: {date-parts: [1957], season: "1990"}` — the fixture repurposes the EDTF `season` slot to smuggle the range's end year (`1990`), and/or the note's `issued: 1957/1990` override should parse as an open interval. Oracle wants `...1957，北京图书馆，1957—[1990]` (em-dash + bracketed end year); citum currently renders only `1957`. Needs either: the note-field `issued` interval parser to handle `YYYY/YYYY`, or a date-component read of the `season` value as an end-year with an em-dash. Touches `crates/csl-legacy/src/csl_json.rs` (note-field date parsing) and/or `crates/citum-engine/src/values/date.rs`, plus the `periodical:` variant in base.yaml (new type-variant added this wave, around line 562).

5. **Map scale + dimensions** (gbt7714.8.13.3:1, 8.13.3:2). `Monograph.scale: Option<String>` already exists in `crates/citum-schema-data/src/reference/types/structural.rs:248` but nothing populates it for map/document types and there's no `scale()` accessor in `accessors.rs`. `dimensions` is currently dropped entirely in `crates/citum-schema-data/src/reference/conversion/scholarly.rs`'s `from_document_ref` (the function fixed this wave for edition/pages — scale/dimensions need the same treatment: read from `legacy_extra_str(&legacy, "scale")` / `"dimensions"`, same pattern as the edition/pages fix). Then: a `scale()` accessor, and two new template components in the `book,thesis,map:` variant — scale as a title-suffix (`. 1:25000`), dimensions as an entry-tail (`. 128cm×84cm`).

6. **CSTR tail dedupe, not a suppression bug — a divergence candidate** (gbt7714.8.14.3:1). Raw fixture: `note: "tex.cstr: 16666.11.nbsdc.tfpbwtqf"` (Zotero's alias spelling) plus a `URL` that happens to contain the same string as part of its path. Citum's csl-legacy parser correctly normalizes `tex.cstr` → canonical `CSTR` (per its own documented design) and the style's `identifier: cstr` component renders it — this is *correct* per Citum's data model. The CSL-M source's own macro (`<if variable="CSTR">...<else>...DOI...</else></if>`) only recognizes the literal `CSTR` variable, not the Zotero `tex.cstr` alias, so citeproc-js's oracle rendering never sees a CSTR value and renders no tail at all. This is a real citeproc-vs-citum divergence (citum arguably more correct), not "citum over-rendering" as previously framed — a registered-divergence candidate for `scripts/report-data/verification-policy.yaml`, not a style fix.

7. **Preprint version prefix** (gbt7714.8.15.2:3, `type: article` with a `version` field). Oracle wants `V2.` rendered before the `arXiv（date）` segment; citum drops it. The `article,dataset,preprint:` variant in base.yaml needs a `variable: version` (or `number: version`) component with a `V` prefix and `.` suffix, positioned before the creation-date group. Check the existing `variable: version, strip-periods: true` component already used elsewhere in base.yaml (flat default template, ~line 152) for the right shape to copy.
