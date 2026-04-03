---
# csl26-tpmn
title: Chicago author-date Chicago 18 rich fidelity follow-up
status: in-progress
type: task
priority: high
created_at: 2026-04-02T11:56:15Z
updated_at: 2026-04-03T17:30:00Z
---

Continue the Chicago author-date fidelity pass using the raw Chicago 18 Zotero
bibliography corpus as supplemental evidence after the primary oracle gate.

Current verified state after the legal-material pass:
- primary oracle remains green at 18 / 18 citations and 32 / 32 bibliography
- official style-scoped rich-input report remains 40 / 40 citations and 53 / 54 bibliography
- Chicago 18 supplemental bibliography benchmark improved from 268 / 406 to
  275 / 401
- APA rich bibliography benchmark improved from 303 / 359 to 306 / 359
- focused legal-material subset improved from 19 / 23 to 20 / 21
- focused `entry-dictionary` / `entry-encyclopedia` subset remains 6 / 11

Policy / architecture constraints established in this pass:
- do not add a top-level `bill` type
- normalize `bill` through existing legal-material paths or generic document
  handling as needed
- keep Chicago 18 as official supplemental evidence, not the hard gate
- preserve the current primary oracle and style-scoped report on every pass

## Iteration checklist

### Pass contract

- [x] Preserve primary oracle at 18 / 18 citations and 32 / 32 bibliography
- [x] Preserve official style-scoped report at 40 / 40 citations and 53 / 54 bibliography
- [x] Improve titleless legal bibliography packaging without introducing a new type
- [ ] Add one more bounded supplemental gain without regressing the hard gates

### Style-only tasks

- [x] Add archive-aware bibliography rendering for `manuscript`
- [x] Add web-native bibliography rendering for `webpage`
- [x] Refine audiovisual bibliography rendering for `broadcast` and
  `motion_picture`
- [x] Improve titleless legal bibliography packaging for `bill` / `legislation`
- [ ] Fix title casing for named legal acts such as `Homeland Security Act`
- [ ] Audit `entry-dictionary` and `entry-encyclopedia` against Zotero output
- [ ] Re-run the official style-scoped report and summarize the next net gain
- [x] Add conversion regression tests that exercise the note-field parser before
  routing legacy references into Citum

## Processor / migration follow-up

- [ ] Normalize note-trapped legal metadata such as `genre: H.R.` and
  `status: enacted` into style-addressable fields (see
  `csl26-bn0r--promote-note-parsed-metadata`)
- [ ] Decide whether titleless `bill` rows should normalize into existing legal
  types or remain generic documents with legal-facing sublabels
- [ ] Fix anonymous/titleless bibliography year-suffix leakage on the Chicago 18
  supplemental corpus (see `csl26-cr7m--chicago-year-suffix-name-order`)
- [ ] Expose a style-addressable legal `code` / container path everywhere
  titleless statute citations need it
- [ ] Preserve `event-title` for `paper-conference` and `speech`
- [ ] Preserve blog reply / medium metadata in a style-addressable field for
  `post-weblog`
- [ ] Expose script writer / cast roles and runtime details for broadcast media

## Root-cause buckets for next pass

- style-defect
  dictionary / encyclopedia authorless container-led packaging
- style-defect
  title casing for named legal acts in bibliography output
- migration-artifact
  legal status / genre / edition details trapped in raw `note`
- processor-defect
  anonymous and titleless year-suffix leakage in supplemental bibliography rows

## Next-round operating model

### Default operating mode

- default to cluster-first co-evolution
- choose exactly one target cluster before editing:
  `entry-dictionary` / `entry-encyclopedia`, named legal-act casing, or
  anonymous/titleless year-suffix leakage
- do not mix clusters in one pass unless the first cluster is exhausted with no
  remaining style-only path

### Evidence ladder

- run the primary oracle first
- run the official style-scoped rich-input report second
- extract only the Chicago 18 supplemental rows for the chosen cluster before
  editing
- treat the full Chicago 18 supplemental corpus as confirmation output, not the
  first debugging surface

### Stop-loss rule

- stop after 2 distinct implementation attempts with no net gain on the chosen
  cluster
- reclassify immediately as `style-defect`, `migration-artifact`, or
  `processor-defect`
- do not continue speculative edits once the cluster looks processor- or
  migration-bound

### Fixture minimization

- create or identify a tiny reproducible subset for the chosen cluster before
  changing style or processor code
- preferred starting shapes:
  2-3 dictionary / encyclopedia rows, 1 named legal-act casing row, or
  2 anonymous/titleless year-suffix rows
- use the reduced fixture set for fast iteration, then confirm on the full
  Chicago 18 supplemental corpus

### Escalation order

- answer the style-only question first: can the mismatch be fixed in YAML with
  current style-addressable data
- only escalate to processor or migration work when the style-only path is
  disproved
- cap workflow/tooling work to at most one direct unlock per pass

### Per-pass tracking fields

- target cluster
- cluster before / after counts
- full supplemental before / after counts
- primary oracle status
- official style-scoped report status
- classification outcome
- stop reason if no net gain is landed

### Strategic alternatives

- processor-first sprint
  use only when the chosen cluster is year-suffix leakage or note-trapped legal
  metadata and the style-only path is already disproved
- fixture-reduction prep pass
  use when the residual mismatch set is still too noisy for efficient debugging;
  extract micro-fixtures and benchmark views only, with no style or schema edits

## Handoff notes

- Treat the Chicago 18 Zotero benchmark as official supplemental evidence, not
  the hard gate.
- Preserve the baseline primary oracle and style-scoped rich-input report while
  narrowing the Chicago 18 gap.
- Group any newly exposed failures by root cause before further edits.
- Keep this bean as the single tracking document for iterative Chicago
  author-date follow-up unless the work clearly splits into separate style and
  processor streams.
