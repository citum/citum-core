---
# csl26-1t2s
title: 'citum-migrate: APA converter produces broken bibliography template'
status: todo
type: bug
priority: normal
created_at: 2026-04-17T23:25:08Z
updated_at: 2026-04-17T23:25:08Z
---

Surfaced during csl26-0ijb triage (2026-04-17). Engine renders APA at 32/32 against embedded `styles/embedded/apa-7th.yaml`, but fresh `citum-migrate styles-legacy/apa.csl` output only scores 16/34.

## Evidence

- `node scripts/oracle.js styles-legacy/apa.csl` → 32/32 (embedded YAML)
- `node scripts/oracle.js styles-legacy/apa.csl --force-migrate` → 16/34
- Direct render of ITEM-25 (Kafka) against embedded YAML: correct
- Same fixture against fresh-migrated YAML: `Kafka, F. 1915. in. Kurt Wolff Verlag` — no title, no parentheses on year, no translator, stray `in.`

## Converter defects (observed in fresh `/tmp/apa-fresh.yaml`)

1. Bibliography template uses heavy `suppress: true` branches with inferred groups (XML-shape retention) instead of declarative components.
2. Date `wrap: parentheses` lost on `date: issued` — year renders bare.
3. Primary title emitted with `suppress: true` at top then re-emitted inside a group — engine dedupe drops it.
4. Translator parenthetical role+wrap not preserved from `<names variable="translator">`.
5. Bare top-level `term: in` / `term: at` components emitted without enclosing group — they render in isolation.
6. Missing type-variants for `book`, `thesis`, `report` etc.; default template shape too weak.

## Scope

`crates/citum-migrate/` — inference and hand-template selection pipeline. Engine side is correct (100% fidelity on embedded YAML; defensive empty-affix suppression landed in csl26-0ijb branch).

## Todo

- [ ] Reproduce fresh-migration bibliography for APA against expanded fixture
- [ ] Trace loss of date `wrap: parentheses` through migrator
- [ ] Trace loss of translator `wrap: parentheses` + role label
- [ ] Audit bare top-level `term:` emission — groups or suppress
- [ ] Fix primary-title double-emit producing silent dedupe
- [ ] Generate type-variants for `book`/`thesis`/`report` so default template isn't universal
