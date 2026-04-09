---
# csl26-5ap9
title: apa authored-containerized works follow-up
status: in-progress
type: task
priority: high
created_at: 2026-04-09T15:40:00Z
updated_at: 2026-04-09T18:45:00Z
---

Own the remaining APA rich bibliography rows after the web-native and
container-packaging clusters are removed.

Current verified state:
- baseline APA gate remains `40 / 40`
- supplemental APA diagnostic benchmark improved from `44 / 74` to `54 / 74`
- this bean initially owns rows `49` to `58` and `60` to `74`
- rows may be split into narrower follow-on beans when a sub-bucket exceeds 3
  rows or spans more than one subsystem

Current sub-buckets:
- audiovisual role / container defects: `49`, `50`, `61`, `63`
- chapter / entry / proceedings defects:
  reduced structural cluster moved from `0 / 10` to `9 / 10`
  remaining local miss: `27a Book chapter`
- conference / presentation classification defects: `67`, `69`, `70`, `73`
- archive / preprint / thesis / artwork / manuscript packaging defects:
  `52`, `53`, `54`, `55`, `58`, `60`, `64`, `65`

Expected owning subsystem:
- mixed `citum_migrate`, `citum_engine`, and style YAML

Current mismatch shape:
- performer / director / studio / label / media details are not packaging like
  APA expects
- chapter / entry / proceedings metadata is now mostly preserved, but one
  editor-translator edge case still needs dedicated cleanup
- conference and presentation rows are inconsistently classified and routed
- archive / thesis / manuscript / artwork rows overuse retrieval-date fallback
  and under-render archive / repository context

Completed in this pass:
- rerouted report chapters with pages + parent title into the component path
- rerouted encyclopedia entries into the collection-component path
- preserved parent edition and container-author metadata for chapter-like rows
- added APA chapter / proceedings / dictionary-entry variants to avoid generic
  retrieval fallback
- added a focused structural regression in
  `crates/citum-engine/tests/bibliography.rs`

Remaining tasks:
- [ ] Isolate and resolve the `27a Book chapter` editor+translator edge case.
- [ ] Split or complete the audiovisual sub-bucket.
- [ ] Split or complete the conference / presentation sub-bucket.
- [ ] Split or complete the archive / thesis / manuscript / artwork sub-bucket.

## Acceptance
- every row currently owned by this bean either matches exactly or is moved to
  a narrower successor bean with an explicit classification and handoff
- baseline APA remains `40 / 40`
- no unknown residuals remain in this bucket

## Stop-Loss Rule
- Stop after 2 distinct implementation attempts per sub-bucket with no net
  gain.
- Reclassify immediately as `style-defect`, `processor-defect`,
  `migration-artifact`, or explicit divergence and record the handoff.
