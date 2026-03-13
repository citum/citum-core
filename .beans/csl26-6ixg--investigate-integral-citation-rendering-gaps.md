---
# csl26-6ixg
title: Investigate integral citation rendering gaps
status: todo
type: bug
priority: high
created_at: 2026-03-13T11:51:00Z
updated_at: 2026-03-13T11:51:00Z
---

## Context

While simplifying `crates/citum-engine/src/processor/rendering.rs` in `csl26-xtt0`, I investigated two nearby behavior gaps:

- explicit multi-item integral citations with an explicit integral template only rendered the first grouped item;
- label-integral fallback behavior appeared to omit the rendered citation label in the narrative form.

## Why This Was Deferred

A direct fix stopped being a bounded simplify change. The attempted implementation widened into grouped-citation assembly and affix/wrap handling, which regressed existing citation behavior across author-date grouping, disambiguation scenarios, and note-style ibid formatting.

Those regressions were broad enough that the safe outcome for the simplify PR was to restore pre-existing grouped rendering semantics and defer the behavior work to a dedicated follow-up.

## Follow-up Checklist

- [ ] Reproduce the current explicit-integral and label-integral gaps with focused tests before changing rendering logic.
- [ ] Define expected grouped integral behavior for multi-item cites with explicit integral templates, including locators and suffixes.
- [ ] Decide whether label-integral fallback should render the citation label, and document the expected narrative form.
- [ ] Implement the behavior change without regressing grouped author-date, disambiguation, or note-style citation output.
- [ ] Verify with `cargo test -p citum-engine --test citations`, targeted `processor::rendering` tests, and `~/.claude/scripts/verify.sh`.
