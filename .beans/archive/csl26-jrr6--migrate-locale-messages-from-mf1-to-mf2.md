---
# csl26-jrr6
title: Migrate locale messages from MF1 to MF2
status: completed
type: feature
priority: high
created_at: 2026-03-22T12:45:24Z
updated_at: 2026-03-22T12:45:24Z
---

Replaced the hand-rolled MF1 evaluator with a dependency-free Mf2MessageEvaluator
supporting ICU MessageFormat 2 syntax. Removed unused mf1-parser crate. Updated
all locale YAML files to MF2 syntax. Removed MessageSyntax::Mf1 variant from types.

Spec: docs/specs/LOCALE_MESSAGES.md
Branch: feat/locale-messages-mf2
Commit: e06919c

## Tasks

- [x] Remove mf1-parser workspace dep from Cargo.toml (unused)
- [x] Introduce MessageEvaluator trait + MessageArgs struct
- [x] Implement Mf2MessageEvaluator with inline evaluation
- [x] Wire evaluator into Locale; update mod.rs default evaluator
- [x] Update locale YAML files (en-US.yaml, de-DE.yaml) to MF2 syntax
- [x] Remove MessageSyntax::Mf1 variant from types.rs
- [x] Update CLI linting and tests for MF2
- [x] Pre-push gate: cargo fmt + clippy + nextest (passed)
- [x] All 8 MF2 message tests passing
- [x] Create deferred ICU4X swap bean (csl26-qrpo — already exists)
