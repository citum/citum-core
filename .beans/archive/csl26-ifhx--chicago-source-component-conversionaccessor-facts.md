---
# csl26-ifhx
title: Chicago source-component conversion/accessor facts
status: scrapped
type: feature
priority: high
created_at: 2026-06-30T14:29:54Z
updated_at: 2026-06-30T18:11:41Z
parent: csl26-40n4
blocked_by:
    - csl26-fr6f
---

Add missing source-component facts to Citum conversion/accessors (crates/citum-migrate + engine accessors): archival correspondence, recordings, performances, broadcasts, original publication dates, event dates, note-derived roles. These facts are shared needs across author-date, notes, shortened-notes, and T&F — the largest remaining fidelity lever per the audit.

## Todo
- [ ] Enumerate exact missing facts and target variant(s) from the audit (csl26-fr6f)
- [ ] Implement accessor/conversion support per fact
- [ ] Add Rust tests per docs/guides/test-coverage conventions
- [ ] Verify fidelity delta via report-core.js

## Reasons for Scrapping

The audit (csl26-fr6f) classified six facts as missing Rust accessor/conversion support: archival correspondence, recordings/broadcasts, original publication dates, event dates, note-derived roles, patent number/issued-date. Direct inspection of crates/citum-schema-data/src/reference/accessors.rs, crates/citum-schema-data/src/reference/conversion/, and crates/citum-migrate/src/upsampler/mapping.rs shows all six already implemented end-to-end (accessor + conversion + migrate mapping):

- Archival: archive/archive_location/archive_name/archive_place/archive_collection accessors (accessors.rs:777-852); migrate maps archive/archive-place/archive_location (upsampler/mapping.rs:602-604).
- Original publication dates: original_date/original_title/original_publisher_str/original_publisher_place (accessors.rs:1332-1349); migrate maps original-date (mapping.rs:655).
- Event dates: parsed + relation_event (conversion/scholarly.rs:381); migrate maps event-date to Variable::EventDate (mapping.rs:652).
- Note-derived roles: ContributorRole::Recipient/Interviewer folded in conversion/mod.rs:46-57, tested in reference/tests.rs:490; migrate maps both.
- Recordings/broadcasts: medium accessor + broadcast handling in fixups/media.rs:186.
- Patent: issued accessor exists; data present.

The audit derived "missing" by inspecting only the Chicago YAML type-variants, not the Rust accessor layer -- it mis-attributed template-usage gaps (a type-variant not referencing an existing accessor) to accessor/conversion gaps. The real gap -- e.g. chicago-author-date-18th's manuscript type-variant not using archive-collection even though archive_collection() exists -- is YAML template wiring, which belongs to csl26-h7oc (drive all Chicago variants to full fidelity), not a Rust conversion bean.

No Rust work proceeds under this bean. csl26-h7oc absorbs the real remaining work: wire existing accessors into Chicago templates where the audit found gaps, then tune fidelity.
