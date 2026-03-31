---
# csl26-ccyy
title: Chicago 18 / APA 8 schema coverage enhancements
status: todo
type: feature
priority: normal
created_at: 2026-03-31T14:45:03Z
updated_at: 2026-03-31T14:56:27Z
---

Implement the schema and migration additions specified in docs/specs/CHICAGO_18_COVERAGE.md.

## Batches

- [ ] Batch 1: Multivolume / serial enrichment fields (volume_title, part_number, part_title, supplement_number, chapter_number)
- [ ] Batch 2: Event top-level type (title, location, date, genre, distributor, organizer, performer, narrator, producer)
- [ ] Batch 3: Status / meta fields (extend status; add available_date, dimensions, scale)
- [ ] Batch 4: Contributor roles (narrator, compiler, producer, composer, performer, host)
- [ ] Batch 5: Review relation — reviewed: Option<Box<InputReference>> on SerialComponent and Monograph; extend section to SerialComponent
- [ ] Batch 6: Original publication — original: Option<Box<InputReference>> on Monograph; deprecate original_date / original_title
- [ ] citum_migrate upsampler: handle all new fields and relations (including container-author → parent.author)
- [ ] coverage-analysis.py reports 0 missing variables on chicago-18th.json

Spec: docs/specs/CHICAGO_18_COVERAGE.md


## APA 8 gaps (from corpus analysis)
- [ ] executive-producer → upsampler maps to producer (no schema change)
- [ ] original-author → upsampler maps to original.author (no schema change)
- [ ] coverage-analysis.py reports 0 missing on apa-7th.json and apa-test.json
