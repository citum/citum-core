---
# csl26-hmwa
title: Tolerant style option structs and citum check --strict
status: todo
type: feature
priority: normal
created_at: 2026-05-16T18:31:28Z
updated_at: 2026-05-16T18:31:28Z
---

Follow-up to csl26-0ksu. The first wave covered the top-level Style + the option structs the forward-compat snapshot exercises (Config, ContributorConfig, Substitute, LocatorConfig). Several nested option structs still hold deny_unknown_fields:

- crates/citum-schema-style/src/options/mod.rs — CitationOptions, BibliographyOptions, BibliographyConfig (lines 159, 240, 343), and the inner ConfigWire (line 656) used by Config's custom Deserialize.
- crates/citum-schema-style/src/options/dates.rs — DateConfig.
- crates/citum-schema-style/src/options/bibliography.rs — 4 structs.
- crates/citum-schema-style/src/options/integral_names.rs — IntegralNameConfig.

Extend the wrapper pattern to each and thread unknown_fields through manual Defaults / constructors. Then add the `citum check --strict` CLI surface noted in csl26-0ksu's original body: when present, re-walk the parsed Style and report any non-empty unknown_fields path as a hard error. Lives in crates/citum-cli/src/commands.rs around the existing run_check function.

Spec: docs/specs/FORWARD_COMPATIBILITY.md (Active).
Tag: forward-compat.
