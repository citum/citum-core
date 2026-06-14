---
# csl26-ya9b
title: 'migrate: spurious URL/accessed + leaked term/type text in bibliography'
status: todo
type: bug
priority: normal
tags:
    - fidelity
    - migrate
created_at: 2026-06-14T11:21:02Z
updated_at: 2026-06-14T17:16:46Z
parent: csl26-vmcr
---

Migrated bibliographies emit fields citeproc omits and leak literal term/type text. china-information: 'Renaissance Art and Culture. Entry encyclopedia. in. Encyclopedia of World History' (leaked 'Entry encyclopedia. in.'); trailing 'accessed' / 'https://...' on non-web types across journal-of-advertising-research, early-medieval-europe. Converter-level: type-template selection emits an entry-encyclopedia literal and unconditional url/accessed. Repro: node scripts/oracle.js styles-legacy/china-information.csl --json --force-migrate

## Root Cause (traced 2026-06-14, deferred)

Not a single converter locus — three distinct defects across two layers, each with its own regression surface:

1. **Leaked `Entry encyclopedia`** — *data-conversion layer*, not citum-migrate. `from_serial_component_ref` injects `genre = "entry-encyclopedia"` from the item type when genre is absent (`crates/citum-schema-data/src/reference/conversion/scholarly.rs:566`). The migrated `entry-encyclopedia` variant (extends `article-newspaper`) renders `variable: genre`, surfacing the injected echo. citeproc never receives that genre, so it renders nothing. The injection has test coverage (citum-schema-data, citum-engine bibliography) — removing it risks regressions.
2. **Leaked `in.`** — template generation: the migrated `article-newspaper` template emits an unconditional `term: in` that fires with no host container.
3. **Spurious url/accessed** — template generation: url/accessed emitted on non-web types.

Entries also show unrelated low-fidelity gaps (missing publisher, dropped volume/pages, patent date format) — compounding defects per the 2026-06-14 locus audit. Deferred from the csl26-vmcr bounded PR for the same reason as csl26-dc1d: cross-layer, multi-cause, not bounded. Repro: `node scripts/oracle.js styles-legacy/china-information.csl --json --force-migrate`.
