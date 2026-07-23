---
# csl26-zaqk
title: Fix nocase HTML markup leaking into plain-text bibliography output
status: completed
type: bug
priority: normal
tags:
    - rendering
    - engine
    - nocase
created_at: 2026-07-23T20:54:40Z
updated_at: 2026-07-23T23:24:04Z
---

Titles authored with Djot [text]{.nocase} case-protection (crates/citum-engine/src/render/rich_text.rs) surface as literal HTML (<span class="nocase">...</span>, <i>...</i>) in the plain-text .text field of citum render --json output, instead of being stripped to plain text. Confirmed in the gb7714-bench v0.77.0 CI artifact (7 occurrences on builtin.json, e.g. entry [161] 'Library of Congress'), longstanding not a recent regression. See docs/architecture/audits/2026-07-23_GB7714_BENCH_COMPARISON.md for examples. Needs a native fixture with a nocase-protected title rendered to plain text.

## Summary of Changes

Root cause was ingestion, not rendering: CSL-JSON titles carry citeproc-js's
literal HTML rich-text convention (`<span class="nocase">`, `<i>`, `<b>`,
`<sc>`, `<sup>`, `<sub>`), and nothing on the CSL-JSON -> `InputReference` path
converted it to Djot (Citum's canonical inline markup) -- so it leaked
verbatim. Djot `[X]{.nocase}` already rendered cleanly before this fix
(`render/rich_text.rs`'s `handle_end_event` already special-cased it); a
renderer-side strip would have thrown away case protection, so the fix
converts at ingestion instead.

- Added `html_markup_to_djot` (+ `DjotTag`/`classify_open_tag`) in
  `crates/citum-schema-data/src/reference/conversion/mod.rs`, converting only
  the fixed citeproc tag set (never a generic `<...>` strip -- titles may
  legitimately contain literal `<`/`>`). Handles nesting
  (`<span class="nocase"><i>X</i></span>` -> `[_X_]{.nocase}`).
- Wired into `build_title` before the colon-split/structuring logic, so title
  + short-title both convert before any downstream heuristics run.
- Scope: titles only (full title, short title, structured parts), per
  discussion -- container-title/publisher/note etc. are out of scope here.
- 6 new unit tests at the conversion boundary (passthrough, nocase, emph/strong,
  nested nocase+emph, unrecognized-tag/bare-angle-bracket preservation, and a
  full `InputReference::from(LegacyReference)` regression test mirroring the
  bean's "Library of Congress" example).

Verified end-to-end against the bean's exact repro (CSL-JSON `webpage` with
`<span class="nocase">Library of Congress</span>` through
`gb-t-7714-2025-numeric`): leaks pre-fix
(`<span class="nocase">Library of Congress</span>[EB]`), clean post-fix
(`Library of Congress[EB]`). No fidelity regression:
`report-core.js --style gb-t-7714-2025-numeric` unchanged at
`fidelityScore: 0.989`, `193/203` GB/T corpus. Full `just pre-commit` gate green
(fmt, clippy `-D warnings`, 2169 nextest tests).

While verifying, found a **separate, pre-existing** bug: native Djot titles
(no CSL-JSON involved) leak raw markup through styles where the title
component resolves `emph: true` (e.g. `apa-7th`'s `titles.monograph.emph`) --
`gb-t-7714-2025-numeric` doesn't set this option and was unaffected. Filed as
follow-up `csl26-d3kj`; not fixed here (out of scope, needs its own root-cause
trace through `render/component.rs`).
