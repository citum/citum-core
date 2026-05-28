---
# csl26-rkmz
title: 'fix: multilingual data-* attributes bypass resolution in extract_metadata'
status: completed
type: bug
priority: normal
created_at: 2026-05-28T13:15:41Z
updated_at: 2026-05-28T13:36:55Z
---

In extract_metadata (grouping.rs:691), data-author is built from authors.to_names_vec() and data-title from reference.title().to_string() — both skip multilingual resolution. The visible bibliography uses resolve_multilingual_name/resolve_multilingual_string with the style's multilingual config (apa-7th: title-mode: combined, name-mode: transliterated). Fix: resolve through the same config so data-* attributes match displayed text.

## Summary of Changes

- Fixed `extract_metadata` in `grouping.rs` to resolve multilingual author/title through `resolve_multilingual_name` / `resolve_multilingual_string` using `bibliography_config.multilingual` settings, matching the visible bibliography render path.
- Added HTML bibliography test asserting `data-author` and `data-title` contain resolved (transliterated/combined) forms for multilingual references.
- Rebuilt release binary and regenerated `docs/demo.html`; `tanaka_yuki2019` entry now shows `data-author="Tanaka, & Suzuki"` and combined transliterated+translated title.
- Enlarged demo page `h1` to `font-size: 2rem; font-weight: 700` in `scripts/build-demo-page.js` (regenerated `docs/demo.html`).
