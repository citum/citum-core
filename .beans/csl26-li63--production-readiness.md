---
# csl26-li63
title: Production Readiness
status: todo
type: milestone
priority: normal
created_at: 2026-02-07T07:40:14Z
updated_at: 2026-04-25T10:57:30Z
---

Tooling, testing infrastructure, documentation, and performance
optimization for the first public release.

## Acceptance criteria

The release goes out when each line below is checked. This list is the
working definition of "production ready" for citum-core; it intentionally
excludes web-platform work (citum-hub) and bindings (citum-labs).

- [x] JSON schema generation for references, citations, locales (csl26-n79w)
- [x] Core vs. Community Style Split (csl26-tb4i)
- [x] Server mode evaluation (csl26-kpv4)
- [ ] Versioning policy doc committed before first public release (csl26-fuw7)
- [ ] Style.version wired to schema validation in `citum check` (csl26-yipx)
- [ ] Style-structure lint hard-fail rollout (csl26-egzd)
- [ ] Dedicated part / supplement / printing number fields (csl26-7edf)
- [ ] User style + locale store (citum_store) — csl26-erwz
- [ ] Djot as default markup for annotations / reference fields (csl26-suz3, in-progress)
- [ ] MaybeGendered<T> on locale terms (csl26-y3kj) — core model is live;
      gender-aware MF2 role-label migration remains follow-up work
- [ ] Gender-aware MF2 role labels with multi-selector `.match` support
      (csl26-vm2g)
- [ ] Locale-backed archive hierarchy labels (csl26-mlc2)

## Strategic pointers

- Roadmap and phase sequencing: [docs/architecture/ROADMAP.md](../docs/architecture/ROADMAP.md)
- Live style fidelity: [docs/TIER_STATUS.md](../docs/TIER_STATUS.md)
- Portfolio gate: `scripts/report-data/core-quality-baseline.json`
- Locale authoring (MF2): [docs/guides/AUTHORING_LOCALES.md](../docs/guides/AUTHORING_LOCALES.md)

## Notes

- The bean body was refreshed 2026-04-25 from a one-liner to the structured
  status above. Children list is authoritative — this body summarises but
  doesn't add new scope. Update the checkboxes when child beans land,
  not the prose.
