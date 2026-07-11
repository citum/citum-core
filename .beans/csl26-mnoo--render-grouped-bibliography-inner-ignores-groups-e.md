---
# csl26-mnoo
title: render_grouped_bibliography_inner ignores groups_enabled
status: todo
type: bug
priority: normal
created_at: 2026-07-11T12:10:07Z
updated_at: 2026-07-11T12:20:24Z
parent: csl26-8m2p
---

Follow-up from a Copilot review comment on PR #1037 (bean csl26-plaz).

`Processor::render_grouped_bibliography_inner`'s custom-groups check
(processor/bibliography/grouping.rs, the historical/pre-PR-1037 branch) tests
only `style.bibliography.groups.is_some()` — it has never checked
`BibliographySpec::groups_enabled`, unlike the correct pattern already used in
`render_selected_bibliography_with_format_and_annotations`
(processor/bibliography/mod.rs:516-522):

```rust
self.style.bibliography.as_ref()
    .filter(|bibliography| bibliography.groups_enabled)
    .and_then(|bibliography| bibliography.groups.as_ref())
```

`groups_enabled` was introduced in bean csl26-rrq9 (2026-05-10, "Suppress
single-group heading + add groups-enabled toggle"), which updated only
`mod.rs`. `grouping.rs`'s `render_grouped_bibliography_inner` was never
updated to match, and has zero test coverage for `groups_enabled` anywhere in
the repo.

Impact: a style author who sets `groups-enabled: false` while still having a
`groups:` block defined (e.g. temporarily disabling grouping without deleting
the config) gets *grouped* rendering instead of the documented "render a flat
bibliography instead" behavior, via:
- the standalone grouped-bibliography API
  (`render_grouped_bibliography_with_format[_and_annotations][_standalone]`)
- FFI `citum_render_bibliography_grouped_*` (`ffi/mod.rs:547-561` — this
  clones the session's *live* run, so it's a real "full library, group-aware"
  view, not dead code)
- CLI, if it ever exposes grouped rendering

(`Processor::render_bibliography`/`render_bibliography_with_format` are a
separate, simpler code path — `process_references_with_format` +
`refs_to_string_with_format`, no grouping awareness at all — and are
unaffected by this bug.)

PR #1037 fixed the analogous new gate added to `render_document_bibliography`
(the `format_document`/`process_document`/`DocumentSession` path) but
deliberately left this pre-existing, separate bug for the standalone API
out of scope.

## Checklist

- [ ] Add the same `.filter(|bibliography| bibliography.groups_enabled)` gate
      to `render_grouped_bibliography_inner`'s custom-groups check
- [ ] Add regression test(s) covering `groups_enabled: false` + `groups:`
      still present via the standalone bibliography API
      (`render_bibliography` / `render_grouped_bibliography_with_format_standalone`)
- [ ] Consider a shared helper (e.g. `Processor::effective_custom_groups(&self) -> Option<&[BibliographyGroup]>`)
      so this condition can't drift between call sites again — three call
      sites currently duplicate the same check inline

Related: csl26-rrq9 (introduced groups_enabled), csl26-plaz (PR #1037,
fixed the analogous new-code instance of this bug).

## Alternative: unify instead of patching in place

Worth considering instead of (or alongside) the minimal `.filter()` patch
above: PR #1037 already introduced `render_flat_document_bibliography`
(grouping.rs), which renders the flat/sort-partitioned-sections cases once
and derives `content` from that render — duplicating, in a corrected form,
the tail of `render_grouped_bibliography_inner` (custom groups aside).
There are now two implementations of "flat or sort-partitioned bibliography
content," one buggy (ignores `groups_enabled`) and one correct.

A unification path: generalize `render_flat_document_bibliography` to accept
the `restrict_to_cited` flag that `render_grouped_bibliography_inner`
already takes (instead of hardcoding cited-only), and have
`render_grouped_bibliography_inner` call it for the flat/partition cases —
discarding the `entries` half when only a content string is needed. Then:

- The `groups_enabled` check only needs to exist in one place (the shared
  gate), removing the "three call sites duplicate the same condition" risk
  called out in the checklist below, by construction rather than by
  convention.
- `render_grouped_bibliography_inner` shrinks to: custom-groups branch
  (unchanged, `render_with_custom_groups_filtered`) + one call into the
  shared flat/partition renderer.

Constraint to preserve: `restrict_to_cited: false` is *not* dead code here
the way it is for `render_document_bibliography` — FFI's
`citum_render_bibliography_grouped_*` (`ffi/mod.rs:547-561`) calls
`render_grouped_bibliography_with_format` against the session's live,
non-empty run with `restrict_to_cited: false`, i.e. a real "render the whole
library, grouped, regardless of what's been cited so far" feature. Any
unification must keep that all-refs behavior working, not just the
cited-only fast path.

`render_with_custom_groups_filtered` (the custom-groups renderer) stays
separate either way — group-local disambiguation and per-group templates
need a different data shape than a flat entries pass, which is exactly why
PR #1037 left custom groups on the historical two-pass render in the first
place.
