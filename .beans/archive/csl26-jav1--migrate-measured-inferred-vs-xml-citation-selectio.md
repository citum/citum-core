---
# csl26-jav1
title: 'migrate: measured inferred-vs-xml citation selection'
status: completed
type: feature
priority: normal
created_at: 2026-06-10T21:16:25Z
updated_at: 2026-06-11T15:14:58Z
parent: csl26-vmcr
---

Follow-up from C4 (bean csl26-3fw0). The template inferrer reports high confidence (0.93) for note-class citation templates that render badly (early-medieval-europe 9/20, zeitschrift-fur-medienwissenschaft 7/20), because its confidence is computed against its own first-position reconstruction surface, not oracle scenarios. Forcing XML-first for all note styles regressed styles whose inferred template was good (6 up / 11 down). Right mechanism: at migration time, render BOTH candidate citation templates (inferred and XML-compiled, both available in the pipeline) against the embedded citeproc-js runtime (js_runtime.rs) on a small scenario set and keep the higher scorer. Evidence: forced XML gave early-medieval-europe 17/20 (vs 9/20 inferred) while chicago-notes-bibliography-access-dates preferred inferred (93%% vs 63%%).

## Summary of Changes

Added measured inferred-vs-XML citation template selection (crates/citum-migrate/src/measured_citation.rs). For note-class styles with an inferred citation template, both candidate standalone styles are assembled, rendered with the Citum engine over the embedded fixture items (bare + page-locator scenarios), and scored against citeproc-js reference renderings using the oracle's bag-of-words Jaccard at 0.60. XML wins only on strictly more passes or a clear similarity margin on ties; any scoring failure keeps inferred. Added citum-engine dependency to citum-migrate and a render_citation_strings entry point to the embedded JS bundle.

Evidence (strict --force-migrate oracle): early-medieval-europe 9/20 -> 17/20 citations; zeitschrift-fur-medienwissenschaft 7/20 -> 20/20; iso690-full-note-es 5/20 -> 13/20; chicago-notes-bibliography-access-dates kept inferred at 20/20 (no regression). Random-100 (seed 20260610): note class at-threshold 15.8% -> 21.1%; overall 52 -> 53 of 100. Shipped in PR #907 (merged 2026-06-11).
