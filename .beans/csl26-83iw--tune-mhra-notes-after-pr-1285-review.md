---
# csl26-83iw
title: 'Tune mhra-notes after PR #1285 review'
status: in-progress
type: task
priority: high
created_at: 2026-10-06T20:08:02Z
updated_at: 2026-10-06T20:53:41Z
parent: csl26-w0hf
---

Resolve review comment pullrequestreview-5433601774 by restoring MHRA bibliography author/title punctuation, then run a bounded style-improvement wave using fidelity, exact-parity, rich-input, and SQI evidence. Update regression coverage, benchmarks/baselines as appropriate, and prepare a review PR.

\n\nEvidence: review regression passes; headline fidelity is 33/34 citations and 43/46 bibliography entries in the current local oracle comparison; SQI baseline remains 0.917; exact parity improved 9/79 -> 27/79. The wave adds journal issue/date structure, monograph/report/thesis/web/broadcast/film/newspaper variants, patent identifiers, URL brackets, and preserves the diagnostic-only MHRA Zotero benchmark (59/127); the embedded parity baseline is ratcheted to 27/79. Remaining exact misses cluster in full-date propagation, localized genre/role wording, repeated-author substitution, and legal/archive field mapping.
