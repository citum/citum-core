---
# csl26-qz8y
title: 'feat(render): add CommonMark/Markdown output format for Pandoc interop'
status: completed
type: feature
priority: normal
created_at: 2026-05-30T19:33:07Z
updated_at: 2026-05-30T19:43:56Z
---

Add OutputFormat::Markdown passthrough that renders citations inline (CommonMark emph/strong) and emits body markup verbatim. Enables pipe workflow: citum render doc --format markdown | pandoc.\n\nAlso reverts the grid-table preprocessor from PR #846 (abandoned branch).\n\n## Tasks\n- [x] Switch to main, close PR #846, create new branch\n- [x] Add crates/citum-engine/src/render/markdown.rs (OutputFormat impl)\n- [x] Wire markdown into render/mod.rs\n- [x] Wire CLI: OutputFormat::Markdown in args.rs, to_document_format, dispatch arms\n- [x] Inline tests in markdown.rs\n- [x] Integration test: pipe table + code block passes through, citations rendered\n- [x] Integration test: note style emits [^n] anchors/definitions with CM emphasis\n- [x] Spec note in docs/ documenting passthrough formats and Pandoc interop\n- [x] Pass pre-commit gate

## Summary of Changes

- Added `crates/citum-engine/src/render/markdown.rs`: new `Markdown` renderer implementing `OutputFormat` with CommonMark inline markup (emph `*x*`, strong `**x**`, small-caps/superscript as raw `<span>`/`<sup>` HTML, semantic passthrough).
- Wired `pub mod markdown` into `render/mod.rs`.
- Added `OutputFormat::Markdown` to CLI `args.rs` enum and `Display` impl.
- Wired five match arms in `commands/render/mod.rs`: `to_document_format`, both `DocumentInput` branches in `render_doc_with_output_format`, `render_refs_human`, `render_refs_json`.
- Two new integration tests in `crates/citum-engine/tests/document.rs`: passthrough of pipe tables/code blocks; note-style footnote syntax emission.
- Spec doc `docs/specs/PANDOC_MARKDOWN_CITATIONS.md` updated (v1.2) with passthrough vs. converted format classification, Pandoc interop workflow, and footnote-extension caveat.
- Grid-table preprocessor from PR #846 does not appear on this branch (branched off clean `main`); PR #846 abandoned.
