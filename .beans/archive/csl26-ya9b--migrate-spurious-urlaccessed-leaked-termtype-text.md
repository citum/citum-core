---
# csl26-ya9b
title: 'migrate: spurious URL/accessed + leaked term/type text in bibliography'
status: completed
type: bug
priority: normal
tags:
    - fidelity
    - migrate
created_at: 2026-06-14T11:21:02Z
updated_at: 2026-06-14T22:25:33Z
parent: csl26-vmcr
blocking:
    - csl26-ivjp
---

Migrated bibliographies emit fields citeproc omits and leak literal term/type text. china-information: 'Renaissance Art and Culture. Entry encyclopedia. in. Encyclopedia of World History' (leaked 'Entry encyclopedia. in.'); trailing 'accessed' / 'https://...' on non-web types across journal-of-advertising-research, early-medieval-europe. Converter-level: type-template selection emits an entry-encyclopedia literal and unconditional url/accessed. Repro: node scripts/oracle.js styles-legacy/china-information.csl --json --force-migrate

## Root Cause (traced 2026-06-14, deferred)

Not a single converter locus — three distinct defects across two layers, each with its own regression surface:

1. **Leaked `Entry encyclopedia`** — *data-conversion layer*, not citum-migrate. `from_serial_component_ref` injects `genre = "entry-encyclopedia"` from the item type when genre is absent (`crates/citum-schema-data/src/reference/conversion/scholarly.rs:566`). The migrated `entry-encyclopedia` variant (extends `article-newspaper`) renders `variable: genre`, surfacing the injected echo. citeproc never receives that genre, so it renders nothing. The injection has test coverage (citum-schema-data, citum-engine bibliography) — removing it risks regressions.
2. **Leaked `in.`** — template generation: the migrated `article-newspaper` template emits an unconditional `term: in` that fires with no host container.
3. **Spurious url/accessed** — template generation: url/accessed emitted on non-web types.

Entries also show unrelated low-fidelity gaps (missing publisher, dropped volume/pages, patent date format) — compounding defects per the 2026-06-14 locus audit. Deferred from the csl26-vmcr bounded PR for the same reason as csl26-dc1d: cross-layer, multi-cause, not bounded. Repro: `node scripts/oracle.js styles-legacy/china-information.csl --json --force-migrate`.

## Summary of Changes

Scoped to defect #1 (the genre echo). Defects #2 (`in.`) and #3 (spurious
url/accessed) are split to follow-up bean csl26-ivjp — both are cross-layer
template-generation changes with a broad regression surface (the original
deferral reason from csl26-vmcr).

**Root-cause correction:** the bean blamed the data-layer genre injection
(`scholarly.rs`), but that injection is *load-bearing* — `ref_type()` reads the
injected genre back to report the entry sub-type for type-variant selection
(e.g. APA's entry-encyclopedia variant). Removing it regressed APA variant
routing. The real leak is the *engine rendering* a genre value that merely
restates the reference's own type.

**Fix (engine, bounded):** `resolve_variable_value` (`citum-engine/src/values/
variable.rs`) now suppresses `SimpleVariable::Genre` when the normalized genre
equals `reference.ref_type()`. Such a genre is the data model's internal
type-carrier (round-tripped for variant selection), which citeproc never emits.
Styles that render real genres (genre != type) are unaffected; APA variant
routing is untouched.

**Verification:** 1605 tests pass; china-information no longer leaks
"Entry encyclopedia." (`Vasari, G. Renaissance Art and Culture. in.
Encyclopedia of World History. 2022`); portfolio gate 154 styles, fidelity 1.0,
0 warnings (no regression).
