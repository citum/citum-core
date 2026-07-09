---
# csl26-nitz
title: Consider Rc→Arc + rayon parallelization for bibliography rendering
status: todo
type: task
priority: deferred
created_at: 2026-07-09T12:59:36Z
updated_at: 2026-07-09T12:59:36Z
parent: csl26-8m2p
---

Once csl26-dog9's FinalizedRun typestate lands (done), Rc<Config>/Rc<BibliographyConfig> (introduced in csl26-qi7l) could become Arc, and Renderer::process_bibliography_entry/render_group_entries could parallelize with rayon, since FinalizedRun makes the read-only-before-render contract typed rather than comment-documented. Out of scope for csl26-dog9 itself. Only worth doing if a real workload shows single-threaded rendering as the bottleneck — the O(n×m) clone cost fixed by csl26-qi7l was the actual hot-path issue, not thread count. See docs/specs/EXPLICIT_RENDER_RUN_STATE.md and the csl26-qi7l follow-up note recorded in csl26-dog9's body.
