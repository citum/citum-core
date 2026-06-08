---
# csl26-tswt
title: 'Resolve Copilot review comments on apply_group_wrap (PR #889)'
status: completed
type: bug
priority: normal
created_at: 2026-06-08T23:31:02Z
updated_at: 2026-06-08T23:31:09Z
---

Two Copilot comments: (1) apply_group_wrap hardcodes ASCII quotes bypassing OutputFormat::wrap_punctuation; (2) only WrapPunctuation captured instead of full WrapConfig (losing inner_prefix/inner_suffix). Fix: capture WrapConfig, pre-compute format-aware wrapped years in render_fallback_grouped_citation_with_format via fmt.inner_affix + fmt.wrap_punctuation, remove apply_group_wrap.

## Summary of Changes

- render_group_item_parts_with_format: return type changed from Option<WrapPunctuation> to Option<WrapConfig>; capture full WrapConfig (preserves inner_prefix/inner_suffix)
- render_fallback_grouped_citation_with_format: pre-compute format-aware wrapped years via fmt.inner_affix + fmt.wrap_punctuation for integral collapsed groups; pass pre_wrapped_years: Option<String> downstream
- build_grouped_citation_content: year_wrap: Option<&WrapPunctuation> → pre_wrapped_years: Option<&str>; uses pre-wrapped string directly when Some
- format_integral_grouped_items: year_wrap param removed; receives already-wrapped content string
- apply_group_wrap: deleted
