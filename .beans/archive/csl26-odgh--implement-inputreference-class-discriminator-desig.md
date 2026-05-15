---
# csl26-odgh
title: Implement InputReference class discriminator design
status: completed
type: task
priority: high
created_at: 2026-05-15T19:00:04Z
updated_at: 2026-05-15T21:10:59Z
blocked_by:
    - csl26-1bdr
---

Implementation of docs/specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md (v0.3).

## Completed change stack

[x] **Layer 1 — schema-data wire discriminator.** `InputReference` has a hand-written flat `class` dispatcher, explicit known-class dispatch for all 18 classes, unknown-class capture, strict known-class payload deserialization, public `ReferenceClass`/`ClassExtension`/`as_<class>()`/`unknown_class()` accessors, and custom schema composition.
[x] **Layer 2 — citum-io constructors and exporters.** BibLaTeX/RIS/CSL JSON conversion paths compile against the shared-base reference model and use `extension()`/`extension_mut()` for class-specific data.
[x] **Layer 3 — citum-migrate constructors.** CSL-to-Citum conversion constructors now build the shared-base `InputReference` through compatibility constructors while tests inspect class data through `extension()`.
[x] **Layer 4 — engine call-site rewrites.** Engine variable/title/sorting/metadata tests and render paths no longer pattern-match on `InputReference` variants; class-specific reads go through `ClassExtension`.
[x] **Layer 6 — forward-compat snapshot.** Row 02b now observes `Pass` for unknown reference classes; row 07 now observes `HardFail` for unknown fields on known reference payloads.
[x] **Layer 7 — schema artifact regeneration.** `docs/schemas/bib.json` regenerated.

## Verification

- `cargo run --bin citum --features schema -- schema --out-dir docs/schemas`
- `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run` — 1282 passed.

Temporary `.ai-intents/` handoff provenance was removed before publication.

Spec: docs/specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md
Parent design bean: csl26-1bdr
