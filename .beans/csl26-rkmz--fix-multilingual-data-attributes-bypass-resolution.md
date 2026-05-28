---
# csl26-rkmz
title: 'fix: multilingual data-* attributes bypass resolution in extract_metadata'
status: in-progress
type: bug
priority: normal
created_at: 2026-05-28T13:15:41Z
updated_at: 2026-05-28T13:30:41Z
---

In extract_metadata (grouping.rs:691), data-author is built from authors.to_names_vec() and data-title from reference.title().to_string() — both skip multilingual resolution. The visible bibliography uses resolve_multilingual_name/resolve_multilingual_string with the style's multilingual config (apa-7th: title-mode: combined, name-mode: transliterated). Fix: resolve through the same config so data-* attributes match displayed text.
