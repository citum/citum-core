---
# csl26-d3kj
title: Djot title markup (e.g. nocase) leaks raw when emph:true bypasses TemplateTitle rendering
status: todo
type: bug
priority: normal
tags:
    - rendering
    - engine
    - title
created_at: 2026-07-23T23:23:29Z
updated_at: 2026-07-23T23:23:50Z
---

Discovered while verifying csl26-zaqk's fix (HTML->Djot conversion at CSL-JSON
ingestion). A NATIVE Citum reference with a Djot title
(`[Library of Congress]{.nocase}`) renders correctly (nocase stripped) through
gb-t-7714-2025-numeric, but leaks the raw markup verbatim through apa-7th when
the title's `TitleRendering.emph` resolves to `true` (apa-7th.yaml sets
`titles.monograph.emph: true`).

Repro (no CSL-JSON involved -- purely native ingestion, isolating this from
csl26-zaqk's ingestion fix):

    citum render refs -b refs.json -s apa-7th -m bib --json
    # refs.json: {"id":"x","class":"monograph","type":"book",
    #             "title":"[Library of Congress]{.nocase}", "issued":{"date-parts":[[2020]]}}
    # => "text": "[Library of Congress]{.nocase}. (2020)."   (should be "Library of Congress. (2020).")

    # Same reference through gb-t-7714-2025-numeric renders correctly:
    # => "Library of Congress[M]. ..."

Root cause not fully isolated. `crates/citum-engine/src/render/component.rs:227-238`
applies `rendering.emph` (a `TemplateComponent`-level rendering flag) to
`component.value` via `fmt.emph(output)` -- but by the time control reaches
there, `component.value` must already be the *raw* unrendered title string
(still containing `[...]{.nocase}` literally), not the Djot-parsed output
`TemplateTitle::values()` (`crates/citum-engine/src/values/title.rs`) produces
in isolation (confirmed correct via the existing
`test_text_case_nocase_protection_in_djot`-style unit tests). Something
upstream of `component.rs` must be bypassing `TemplateTitle::values()`'s
Djot-aware path specifically when an emph/strong/small-caps/superscript
rendering flag is set on the title component -- needs tracing from wherever
`TemplateComponent`/`ProcValues` get constructed for title components down to
`component.rs`'s `render_generic_component`-equivalent, to find where the raw
field value is substituted for the rendered one.

Affects any style that applies `emph`/`strong`/`small_caps`/`vertical_align`
directly to a title component (at minimum `apa-7th`; other embedded styles
with per-type title emphasis options may share the bug -- worth a broader
scripted check once root-caused). Independent of csl26-zaqk's CSL-JSON HTML
conversion; this reproduces with hand-authored native Djot titles.
