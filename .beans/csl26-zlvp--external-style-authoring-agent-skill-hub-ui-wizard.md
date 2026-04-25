---
# csl26-zlvp
title: 'External style authoring: agent skill + hub UI wizard'
status: todo
type: feature
priority: deferred
tags:
    - style
    - dx
created_at: 2026-03-16T20:11:54Z
updated_at: 2026-04-25T20:20:07Z
blocked_by:
    - csl26-fuw7
---

Track the work to expose Citum style authoring to average users via (a) a self-contained agent skill and (b) a citum-hub paste-references UI wizard. Blocked on schema stability (v1.0) and stable example corpus.

## Background

`style-evolve` is internal: it knows repo paths, the oracle, migration scripts. An external version needs to be self-contained and agent-agnostic (works in Claude.ai, Copilot, GPT, etc.).

`docs/schemas/style.json` is already published — the raw material exists. What's missing is a stable schema version, annotated examples, and (for hub) a render API endpoint.

## Scope

### Phase 1 — Prerequisites (blocking everything else)
- [ ] Schema reaches v1.0 stability (see csl26-fuw7 versioning policy)
- [ ] 3–5 canonical example styles with inline YAML comments explaining template logic
- [ ] Self-contained style authoring guide (no internal repo knowledge required)

### Phase 2 — External agent skill (`style-author`)
A standalone skill (not tied to this repo) that an average user can install in any agent:
- CSL → Citum conversion: reads the CSL XML + schema, produces Citum YAML
- HTML/PDF → Citum: user points at formatted reference examples; agent reverse-engineers the template logic
- Bundles: schema URL, example styles, authoring guide as references/
- Publishes: to a public GitHub repo (e.g. citum-org/style-author-skill)
- Promotes: from docs/index.html and the static docs site

### Phase 3 — citum-hub UI wizard
UI component in citum-hub where a user pastes 5–10 formatted references and an LLM generates the Citum YAML template. Requires:
- A stable /render API endpoint in citum-hub (separate repo work)
- The Phase 2 skill or equivalent prompt chain as the backend logic
- Live preview: paste reference → rendered output side-by-side

### Phase 4 — Promotion
- Call-to-action on docs/index.html ('Create a style in minutes')
- Interactive demo page (docs/interactive-demo.html already exists — extend it)
- Announce in citum-org community channels

## Design Notes

The external skill should use `docs/schemas/style.json` (hosted URL) as its primary context artifact — not inline schema content. This keeps the skill lightweight and always up-to-date.

For multi-agent compatibility: skill body should be pure Markdown instructions with no Claude-specific tool assumptions. Inline examples are better than tool-call sequences.

The hub wizard idea is the highest user-value item but also the most work (needs API + frontend + LLM plumbing). Phase 2 (standalone skill) is the right first step and is independently useful.
