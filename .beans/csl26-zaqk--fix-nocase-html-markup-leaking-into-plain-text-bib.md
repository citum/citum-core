---
# csl26-zaqk
title: Fix nocase HTML markup leaking into plain-text bibliography output
status: todo
type: bug
priority: normal
created_at: 2026-07-23T20:54:40Z
updated_at: 2026-07-23T20:54:40Z
---

Titles authored with Djot [text]{.nocase} case-protection (crates/citum-engine/src/render/rich_text.rs) surface as literal HTML (<span class="nocase">...</span>, <i>...</i>) in the plain-text .text field of citum render --json output, instead of being stripped to plain text. Confirmed in the gb7714-bench v0.77.0 CI artifact (7 occurrences on builtin.json, e.g. entry [161] 'Library of Congress'), longstanding not a recent regression. See docs/architecture/audits/2026-07-23_GB7714_BENCH_COMPARISON.md for examples. Needs a native fixture with a nocase-protected title rendered to plain text.
