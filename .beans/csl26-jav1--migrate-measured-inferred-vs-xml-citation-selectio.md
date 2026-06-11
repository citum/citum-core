---
# csl26-jav1
title: 'migrate: measured inferred-vs-xml citation selection'
status: in-progress
type: feature
priority: normal
created_at: 2026-06-10T21:16:25Z
updated_at: 2026-06-11T09:53:02Z
parent: csl26-vmcr
---

Follow-up from C4 (bean csl26-3fw0). The template inferrer reports high confidence (0.93) for note-class citation templates that render badly (early-medieval-europe 9/20, zeitschrift-fur-medienwissenschaft 7/20), because its confidence is computed against its own first-position reconstruction surface, not oracle scenarios. Forcing XML-first for all note styles regressed styles whose inferred template was good (6 up / 11 down). Right mechanism: at migration time, render BOTH candidate citation templates (inferred and XML-compiled, both available in the pipeline) against the embedded citeproc-js runtime (js_runtime.rs) on a small scenario set and keep the higher scorer. Evidence: forced XML gave early-medieval-europe 17/20 (vs 9/20 inferred) while chicago-notes-bibliography-access-dates preferred inferred (93%% vs 63%%).
