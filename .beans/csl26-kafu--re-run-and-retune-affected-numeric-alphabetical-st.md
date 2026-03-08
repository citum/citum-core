---
# csl26-kafu
title: Re-run and retune affected numeric alphabetical styles after semantic sort fallback
status: todo
type: task
priority: high
created_at: 2026-03-08T00:33:31Z
updated_at: 2026-03-08T00:33:31Z
---

The missing-name sort fallback change is global enough to improve one class of
alphabetical styles while subtly perturbing others. After the engine-side fix,
the affected numeric alphabetical styles need a fresh oracle pass and targeted
retuning so we do not trade one ordering bug for another family-specific
regression.

Re-run the relevant styles, compare pre/post ordering for anonymous and
missing-contributor works, and adjust only the styles that still disagree with
their authority basis. Keep the focus on the numeric alphabetical cohort first,
then capture any shared remediation patterns that should move into presets or
engine logic.
