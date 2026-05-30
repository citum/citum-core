---
# csl26-7ld5
title: 'Resolve Copilot review comments on PR #840'
status: in-progress
type: task
priority: high
created_at: 2026-05-30T01:54:15Z
updated_at: 2026-05-30T01:54:15Z
---

Fix four issues raised by Copilot on PR #840: (1) Djot verbatim text escaped via push_text instead of push_raw_text; (2) Markdown Event::Code pre-escapes content before inline_code; (3) Typst code_block uses fixed backtick fence; (4) Typst inline_code uses single backtick. Also: LaTeX inline_code must self-escape once callers stop pre-escaping.
