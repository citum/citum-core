---
# csl26-o33x
title: Thread locale quote terms through rendering
status: completed
type: task
priority: normal
tags:
    - localization
    - rendering
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-05T19:49:54Z
parent: csl26-8m2p
---

Locale open/close/inner quote terms exist in the schema (en-US and others) but every backend hardcodes English marks (unicode_quote_marks, per-backend quote()/wrap_punctuation, LaTeX backtick pairs). A fr-FR style declaring guillemets renders curly quotes everywhere. Thread the active locale quote terms through quote_marks(depth); keep hardcoding only as fallback. Needs locale access at the OutputFormat boundary — design first. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 12.

## Summary of Changes

- Widened the `OutputFormat` trait (`quote`, `quote_marks`, `quote_with_depth`, `wrap_punctuation`) to accept a new `QuoteMarks` value (open/close/inner pair), replacing the hardcoded `unicode_quote_marks()` fallback used by every backend.
- Added `QuoteMarks` in `render/format.rs` with a `Default` matching the old English curly-quote fallback and a `From<&GrammarOptions>` conversion from the locale's already-existing `open_quote`/`close_quote`/`open_inner_quote`/`close_inner_quote` fields.
- Updated all backends (html, latex, markdown, djot, org, plain, typst) to render the passed-in marks instead of hardcoded characters. LaTeX's backtick-pair override (``` `` ``/`''`) was removed entirely per user decision, so LaTeX now honors the real locale marks like every other backend.
- Added `quote_marks: QuoteMarks` to `ProcTemplateComponent` (mirroring the existing `item_language` field) so the resolved locale marks travel from `RenderOptions.locale` at component-construction time (grouped/core.rs, values/list.rs, values/message.rs) down to the render call, with no signature changes needed on the widely-used `render_component_with_format`/`render_component_with_format_and_renderer` wrappers.
- Threaded real locale marks at the other direct `wrap_punctuation`/`quote` call sites that already had `self.locale` or `options.locale` in scope (processor/citation.rs, processor/rendering/grouped/core.rs, values/contributor/substitute.rs). The djot/org inline rich-text quote helpers (`render/rich_text.rs`) keep the Unicode default fallback since no locale is threaded through that subsystem yet.
- Added/updated unit tests exercising guillemets and other non-default `QuoteMarks` alongside the existing English-default cases so the fallback path isn't the only one under test.

Full workspace `just pre-commit` (fmt + clippy + nextest) passes: 1793/1793 tests green.
