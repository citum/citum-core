# Feature Metadata

User-visible feature maturity is tracked in
[`features.yaml`](features.yaml). The file is a docs-facing registry, not a
runtime compatibility mechanism.

Runtime compatibility remains enforced by the schema and engine:

- `info.citum-version` declares the minimum Citum engine requirement for a
  style.
- style schema major/minor checks report incompatible or possibly unsupported
  schema versions.
- `extends-pin` locks inherited remote styles by content identifier.
- `citum style validate` runs schema validation, inheritance resolution,
  pin verification, and `citum-version` checks.

The registry adds a stable place for docs pages and examples to show when a
feature became available:

- `since_schema`: minimum style schema version for the documented surface
- `since_engine`: minimum engine version for the documented behavior
- `status`: `active`, `preview`, `experimental`, or `planned`
- `spec`: optional source document for the underlying design or policy

The first pass is warning-only. Missing metadata should be fixed as docs are
touched, then promoted to a stricter build gate once coverage is complete.
