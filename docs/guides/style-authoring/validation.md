---
title: Validation and Compatibility
nav: Validation
description: Check styles before publishing and understand the existing version enforcement boundaries.
features:
  - style-authoring
  - style-inheritance-pins
---

## [fact_check] Validate before publishing

Run the end-to-end style validator before publishing or comparing fidelity.

```bash
citum style validate path/to/my-journal.yaml
```

The validator parses the style, runs schema validation, resolves `extends`,
verifies any `extends-pin` values, and checks `info.citum-version` against the
running engine.

Machine-readable output is available for tools:

```bash
citum style validate path/to/my-journal.yaml --format json
```

## [upgrade] Engine compatibility

Use `info.citum-version` when a style depends on an engine feature that older
versions should not try to render.

```yaml
info:
  title: Hypothetical Style With New Features
  citum-version: ">=0.49.0"
```

An older engine returns a direct version mismatch instead of an opaque
deserialization or rendering failure.

## [schema] Schema compatibility

Style schema versioning and engine versioning are related but separate:

- schema version describes the YAML wire format
- engine version describes the Rust processor release
- generated schemas in `docs/schemas/` remain the committed validation
  artifacts

For policy details, see [`SCHEMA_VERSIONING.md`](../../reference/SCHEMA_VERSIONING.md).

## [analytics] Fidelity reports

Use the generated reports when checking whether a style is production-ready:

- [`compat.html`](../../compat.html) for style-level fidelity and SQI status
- [`reports.html`](../../reports.html) for generated report entrypoints
- [`TIER_STATUS.md`](../../TIER_STATUS.md) for the current style status table
