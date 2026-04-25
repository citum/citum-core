---
# csl26-li63
title: Production Readiness
status: todo
type: milestone
priority: normal
created_at: 2026-02-07T07:40:14Z
updated_at: 2026-04-25T20:24:48Z
---

Tracks remaining blockers for citum-core 1.0. Excludes web-platform
work (citum-hub) and bindings (citum-labs).

## Acceptance Criteria

### Schema (land in sequence — freeze after all checked)

- [ ] `RichText` type for `note`/`abstract` (csl26-suz3, in-progress) — step 1
- [ ] Gender-aware MF2 role labels + multi-selector `.match` (csl26-vm2g) — step 2, with mlc2
- [ ] Locale-backed archive hierarchy labels (csl26-mlc2) — step 2, blocked by csl26-y3kj
- [ ] Dedicated part/supplement/printing-number fields (csl26-7edf) — step 3
- [ ] `Style.version` wired to schema validation (csl26-yipx) — step 4, co-land with csl26-fuw7
- [ ] `Place` type unification: archive vs publisher (csl26-iphj) — deferred, candidate for 1.1

### Other blockers

- [ ] Versioning policy doc (csl26-fuw7) — co-land with csl26-yipx
- [ ] MaybeGendered snapshot tests (csl26-y3kj) — core model live; tests only
- [ ] User style + locale store format config (csl26-erwz)
- [ ] Style-structure lint hard-fail rollout (csl26-egzd)

### Done

- [x] JSON schema generation for references, citations, locales (csl26-n79w)
- [x] Core vs. Community Style Split (csl26-tb4i)
- [x] Server mode evaluation (csl26-kpv4)

## Strategic Pointers

- Roadmap and phase sequencing: [docs/architecture/ROADMAP.md](../docs/architecture/ROADMAP.md)
- Live style fidelity: [docs/TIER_STATUS.md](../docs/TIER_STATUS.md)
- Portfolio gate: `scripts/report-data/core-quality-baseline.json`
- Locale authoring (MF2): [docs/guides/AUTHORING_LOCALES.md](../docs/guides/AUTHORING_LOCALES.md)
