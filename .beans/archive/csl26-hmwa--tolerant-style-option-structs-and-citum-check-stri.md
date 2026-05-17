---
# csl26-hmwa
title: Tolerant style option structs and citum check --strict
status: completed
type: feature
priority: normal
created_at: 2026-05-16T18:31:28Z
updated_at: 2026-05-17T23:20:54Z
---

Follow-up to csl26-0ksu. The first wave covered the top-level Style + the option structs the forward-compat snapshot exercises (Config, ContributorConfig, Substitute, LocatorConfig). Several nested option structs still hold deny_unknown_fields:

- crates/citum-schema-style/src/options/mod.rs — CitationOptions, BibliographyOptions, BibliographyConfig (lines 159, 240, 343), and the inner ConfigWire (line 656) used by Config's custom Deserialize.
- crates/citum-schema-style/src/options/dates.rs — DateConfig.
- crates/citum-schema-style/src/options/bibliography.rs — 4 structs.
- crates/citum-schema-style/src/options/integral_names.rs — IntegralNameConfig.

Extend the wrapper pattern to each and thread unknown_fields through manual Defaults / constructors. Then add the `citum check --strict` CLI surface noted in csl26-0ksu's original body: when present, re-walk the parsed Style and report any non-empty unknown_fields path as a hard error. Lives in crates/citum-cli/src/commands.rs around the existing run_check function.

Spec: docs/specs/FORWARD_COMPATIBILITY.md (Active).
Tag: forward-compat.


## Summary of Changes

Extended the capture-unknown-fields wrapper pattern to the nine remaining
nested style-option structs that still held `deny_unknown_fields`:

- `crates/citum-schema-style/src/options/mod.rs` — `CitationOptions`,
  `BibliographyOptions`, `NoteConfig`.
- `crates/citum-schema-style/src/options/dates.rs` — `DateConfig`.
- `crates/citum-schema-style/src/options/bibliography.rs` —
  `BibliographyConfig`, `ArticleJournalBibliographyConfig`,
  `BibliographySortPartitioning`, `CompoundNumericConfig`.
- `crates/citum-schema-style/src/options/integral_names.rs` —
  `IntegralNameConfig`.

The inner `ConfigWire` already captured unknowns from the 0ksu wave; no
change needed there.

### CLI surface

Added `citum check --strict`. The shared walker lives at
`citum_engine::api::collect_unknown_field_paths`; the engine integration
test in `forward_compatibility.rs` now delegates to it so tests and CLI
share a single source of truth.

### Producer-side validation

Loosening the Rust loader removed `additionalProperties: false` from
`docs/schemas/*.json`, which would have lost editor typo-catching for
producers. The schema CLI now post-processes each generated document and
stamps `unevaluatedProperties: false` on every tolerant type
(`crates/citum-cli/src/commands/schema.rs`), including the inlined
`InputReference` discriminator branches. After regeneration:
`style.json` carries 20 stamps; `bib.json` carries 19 — restoring the
producer-side contract called out in `csl26-0ksu`'s commit message.

### Spec

`docs/specs/FORWARD_COMPATIBILITY.md` flagged provisional pending 1.0
review; behaviour unchanged.
