---
# csl26-odgh
title: Implement InputReference class discriminator design
status: completed
type: task
priority: high
tags:
    - forward-compat
created_at: 2026-05-15T19:00:04Z
updated_at: 2026-05-16T12:52:31Z
blocked_by:
    - csl26-1bdr
---

Implementation of `docs/specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md` (v0.4).

This bean is no longer a Layer 1-only checkpoint. The live implementation now uses the flat `class` dispatcher for `InputReference`, keeps class payloads as the resident storage, and exposes shared bibliographic fields through accessors rather than moving them onto public outer-struct fields. This reconciles the original v0.3 design with the implementation that actually landed.

## Completed in this stack

- [x] Flat `InputReference` deserialize/serialize boundary for all 18 known classes.
- [x] Unknown top-level classes capture into `UnknownClassData` and round-trip without wrapper leakage.
- [x] `ReferenceClass`, `ClassExtension`, `extension()`, `extension_mut()`, `as_<class>()`, and `unknown_class()` accessors are live.
- [x] `deny_unknown_fields` is restored across reference payload structs.
- [x] JSON Schema emits per-class `class` const branches with `unevaluatedProperties: false`.
- [x] `citum-io`, `citum-migrate`, and engine call sites compile against the new accessor/extension model.
- [x] Document formatting emits `unknown_reference_class` warnings with `ref_id` and class string.
- [x] Forward-compat row `02b-discriminator-class` is `declared=SoftDegrade observed=SoftDegrade`.
- [x] Schema/discriminator alignment coverage asserts known-class strictness and the intentional unknown-class schema/engine asymmetry.

## Deliberate remainder

- Row `07-new-reference-field` remains a separate forward-compat gap under `csl26-acfh`: strict known-class payloads now reject unknown fields instead of silently dropping them, but the broader tolerant-reference-field warning path is not part of this discriminator bean.
- The broader tolerant enum/style-option/locale follow-ups remain tracked under `csl26-ld6e`, `csl26-0ksu`, and `csl26-o1z5`.

## Verification target

Run the Rust pre-commit gate before publishing:

```bash
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run
```

Spec: `docs/specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md`
Forward-compat spec: `docs/specs/FORWARD_COMPATIBILITY.md`
Parent design bean: `csl26-1bdr`
