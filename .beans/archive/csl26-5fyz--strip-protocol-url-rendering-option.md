---
# csl26-5fyz
title: Strip-protocol URL rendering option
status: completed
type: task
priority: low
tags:
    - schema
    - rendering
created_at: 2026-07-12T18:16:02Z
updated_at: 2026-07-12T22:10:57Z
parent: csl26-kcda
---

CSL schema#395: no strip-protocol/unprefixed-URL rendering option on
LinksConfig ({doi, url, target: LinkTarget, anchor: LinkAnchor}). MLA
8th edition style requires omitting http://https:// when displaying URLs.

- [x] Add a strip-protocol rendering option to LinksConfig

## Summary of Changes

Added `strip_protocol: Option<bool>` to `LinksConfig` (crates/citum-schema-style/src/options/mod.rs). Wired into `resolve_url` (crates/citum-engine/src/values/mod.rs): when `Some(true)`, strips a leading `https://` or `http://` from the resolved link string. `resolve_effective_url` needed no changes — it already forwards the matched `LinksConfig` into `resolve_url`. Added 4 unit tests covering unset/https/http/explicit-false cases. Regenerated docs/schemas/style.json via `just schema-gen`.
