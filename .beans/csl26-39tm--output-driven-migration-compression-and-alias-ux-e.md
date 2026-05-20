---
# csl26-39tm
title: Output-driven migration compression and alias UX evidence
status: in-progress
type: feature
priority: high
created_at: 2026-05-20T19:32:13Z
updated_at: 2026-05-20T22:16:47Z
parent: csl26-f1u7
blocked_by:
    - csl26-kqji
---

Follow-up split from PR2 after the XML/AST medoid compaction attempt proved fidelity-risky.

Scope:
- Build structural compression from output-driven inference/oracle evidence, not by flattening parsed CSL XML branches.
- Target apa-6th-edition specifically: reduce migrated output below 1,500 LOC without treating APA 6th as an alias of APA 7th and without bibliography/citation oracle regression.
- Emit machine-readable evidence for future UX: exact registry alias status, parent/template link, canonical target, emitted form, preserved deltas, discarded deltas, and output-size reduction.
- Design optional UI choices around evidence-backed actions: keep standalone, register local alias, or propose global alias. Global aliases require reviewed equivalence, not ancestry/template evidence.
- Revisit long-tail styles such as american-medical-association and institute-of-physics-numeric only where output-driven deltas prove the compression is behavior-preserving.
