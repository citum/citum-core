---
# csl26-7ld5
title: 'Resolve Copilot review comments on PR #840'
status: completed
type: task
priority: high
created_at: 2026-05-30T01:54:15Z
updated_at: 2026-05-30T01:57:32Z
---

Fix four issues raised by Copilot on PR #840: (1) Djot verbatim text escaped via push_text instead of push_raw_text; (2) Markdown Event::Code pre-escapes content before inline_code; (3) Typst code_block uses fixed backtick fence; (4) Typst inline_code uses single backtick. Also: LaTeX inline_code must self-escape once callers stop pre-escaping.

## Summary of Changes

- renderer.rs: renamed in_code_block to in_raw_context (covers CodeBlock + InlineCode)
- djot.rs: use in_raw_context so verbatim span text is not escaped
- markdown.rs: pass raw code to inline_code (no fmt.text() pre-escape); use in_raw_context for Text events
- typst.rs: added longest_backtick_run helper; dynamic fence for code_block (max(run,2)+1 ticks) and inline_code (run+1 ticks)
- latex.rs: inline_code now calls self.text() internally since callers no longer pre-escape
