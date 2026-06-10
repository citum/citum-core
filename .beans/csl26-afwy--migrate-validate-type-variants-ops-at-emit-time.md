---
# csl26-afwy
title: 'migrate: validate type-variants ops at emit time'
status: todo
type: bug
priority: high
created_at: 2026-06-10T16:56:45Z
updated_at: 2026-06-10T16:56:45Z
parent: csl26-vmcr
---

Cluster C1 from docs/architecture/audits/2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md. citum-migrate emits bibliography.type-variants operations that reference components absent from the base template; the processor hard-fails the entire style at render (template variant operation matched no component). Evidence: zeitschrift-fur-fantastikforschung (interview), american-mathematical-society-label (patent). Fix in crates/citum-migrate: validate variant ops against the emitted base template; drop or repair invalid ops and record the decision in the evidence sidecar. Add regression tests. Converter must never emit YAML the processor rejects.
