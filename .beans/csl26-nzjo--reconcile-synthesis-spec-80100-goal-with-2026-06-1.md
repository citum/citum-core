---
# csl26-nzjo
title: Reconcile synthesis spec 80/100 goal with 2026-06-11 stop decision
status: todo
type: task
priority: low
tags:
    - migrate
    - docs
created_at: 2026-07-17T15:43:07Z
updated_at: 2026-07-17T16:02:38Z
---

OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md still carries '>80/100 at >=90% combined' as an acceptance criterion, while the 2026-06-11 wave outcome stopped improvement waves on that number and csl26-hxhx (XML layout compiler removal gate) remains open. Re-scope the 80/100 line as a long-horizon converter-bugfix aspiration, not a gate, and cross-link the audits. Context: docs/architecture/audits/2026-07-17_MIGRATION_APPROACH_STRATEGIC_REVIEW.md

## Tasks

- [ ] Edit docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md: re-scope the '>80/100 at >=90% combined' acceptance criterion as a long-horizon converter-bugfix aspiration, not a gate; cross-link the 2026-06-11 wave outcome and the 2026-07-17 strategic review
- [ ] Confirm crates/citum-migrate/CLAUDE.md language matches (converter-dominated tail, no active improvement waves)
- [ ] Re-check the csl26-hxhx removal gate: run the seeded scorecard with selection debug and record whether the xml seed still wins selections; update csl26-hxhx with the current count
