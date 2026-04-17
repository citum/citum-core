---
# csl26-0ijb
title: APA bib engine-level rendering gaps
status: todo
type: bug
priority: normal
created_at: 2026-04-17T18:40:49Z
updated_at: 2026-04-17T18:40:49Z
---

## Context

Surfaced during `/migrate-research` cycle on 2026-04-17 after PR #532 fixed
`name-as-sort-order` mapping in the converter. Running
`node scripts/oracle.js styles-legacy/apa.csl --force-migrate` still shows
16/34 bibliography entries passing. The remaining 18 failures are
engine-level (processor-defect), not converter issues.

## Failure patterns

1. **Spurious `in.` / `in` token** between components (15+ entries) —
   appears where no container/parent-serial is rendered but an "in" connector
   leaks into output.
2. **Missing titles** — migrated YAML contains correct `title` components
   (`/tmp/apa-fresh.yaml` has 65 title declarations across types), but engine
   omits them from bibliography output for many types.
3. **"personal communication" leaks** into unrelated types (chapter, video
   interview — entries 30, 31).
4. **Missing translator parenthetical** `(D. Wyllie, Trans.)` (entry 17 Kafka).
5. **Container-author rendering** broken — "Container-authorS. Colbert
   (Interviewer)" (entry 31).

## Evidence

- Fresh migration YAML `/tmp/apa-fresh.yaml` declares titles and translator
  components correctly.
- Oracle diff entries 17–34 show the patterns above; converter output is
  structurally correct, rendering is wrong.

## Scope

Engine-level investigation in `crates/citum-engine/`. Not a converter task.
Likely overlaps with known `in.` prefix bug already in session memory.

## Todo

- [ ] Reproduce entry 17 (Kafka translator) in engine unit test
- [ ] Identify source of spurious `in` token
- [ ] Audit title rendering path for APA bibliography templates
- [ ] Trace personal-communication leak into non-personal-comm types
- [ ] Verify container-author rendering for video-interview type
