---
# csl26-awef
title: Replace global panic-hook swap in candidate scoring
status: todo
type: task
created_at: 2026-07-06T18:42:20Z
updated_at: 2026-07-06T18:42:20Z
parent: csl26-al39
---

Audit F3 (2026-07-06 migrate review): measured_citation.rs::catch_candidate_unwind swaps the process-global panic hook around every bibliography-candidate render. Latent race under any future parallelism, and the panic payload is discarded so engine bugs are indistinguishable from bad candidates (scored 0). Fix: install a silencing hook once via std::sync::Once, or keep catch_unwind and tracing::debug! the captured payload with the candidate name.
