---
# csl26-7yfp
title: Finalize delimiter migration strategy
status: completed
type: feature
priority: high
created_at: 2026-02-07T12:11:53Z
updated_at: 2026-02-27T14:23:21Z
parent: csl26-u1in
---

Resolve and document the delimiter migration approach.

Options under consideration:
- Hybrid enum (Some(Delimiter::Comma) vs Some(Delimiter::Custom("...")))
- Simple string (Option<String>)
- Trade-offs: Type safety vs flexibility

Deliverables:
- Documented decision in MEMORY.md or design doc
- Implementation complete
- Migration guide for style authors

Refs: csl26-6bak

## Summary of Changes

Decision resolved through implementation. The hybrid enum approach (csl26-6bak, commit 697d083) was adopted: `DelimiterPunctuation::from_csl_string` in schema serves as the single source of truth. Migrate and engine both normalized to use the enum. No separate documentation doc written — the implementation is the canonical strategy.
