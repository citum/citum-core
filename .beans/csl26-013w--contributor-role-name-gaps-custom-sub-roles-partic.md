---
# csl26-013w
title: Contributor role & name gaps (custom sub-roles, particle abbreviation)
status: todo
type: feature
priority: low
tags:
    - schema
    - contributors
created_at: 2026-07-12T15:35:37Z
updated_at: 2026-07-12T16:02:13Z
parent: csl26-kcda
---

- No mechanism for a custom/arbitrary per-contributor role label beyond the
  closed role enums (e.g. "cartographer", "speaker") — CSL schema#361.
  Note: style.json's ContributorRole enum is richer (22 values, includes
  editorial-director/textual-editor/collection-editor/curator/etc.) than
  bib.json's data-input ContributorRole (15 values) — worth checking whether
  any of #361's asks are already reachable through that gap before adding
  a fully open custom-role mechanism.
- No configurable rule for automatically abbreviating name particles on
  render (e.g. German "von" -> "v.") — CSL schema#424. Particle storage
  itself (dropping-particle/non-dropping-particle) already exists; this is
  about a render-time abbreviation rule, and DemoteNonDroppingParticle
  (sort-only vs display-and-sort) doesn't cover it either.

- [ ] Design: custom sub-role mechanism, informed by the bib.json/style.json
      ContributorRole gap noted above
- [ ] Design: particle-abbreviation rendering rule (locale- or style-driven)
