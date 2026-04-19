---
# csl26-b4h2
title: Script discovery of hidden parent-style alias candidates
status: todo
type: feature
priority: normal
created_at: 2026-04-19T12:33:36Z
updated_at: 2026-04-19T12:33:36Z
---

Build `scripts/find-alias-candidates.js` that identifies independent CSL styles rendering identically (or near-identically) to an existing builtin parent — surfacing hidden alias opportunities.

## Motivation
CSL has no requirement that a journal using e.g. T&F Chicago Author-Date be marked as a dependent of that style. Many journals are submitted as standalone independent styles even when they follow a well-known parent's rules exactly (e.g. Annals of the Association of American Geographers → taylor-and-francis-chicago-author-date, aliased in commit after csl26-28g0).

URL-scanning alone is insufficient: AAG's CSL never mentions tandfonline. Behavioral fingerprinting via rendered output is the reliable signal.

## Approach
For each candidate independent style in `styles-legacy/`:
1. Render the strict 12-scenario fixture via citeproc-js
2. Compare output against each registry builtin target (or curated parent list)
3. Report similarity score per (candidate, target) pair
4. Output TSV sorted by confidence

Reuse `scripts/oracle.js` rendering plumbing; no new citeproc integration needed.

## Tasks
- [ ] Draft `scripts/find-alias-candidates.js` (Node, uses citeproc-js like oracle.js)
- [ ] Define similarity function (citation + bibliography string equality per scenario)
- [ ] Curate target list (start with registry builtins — `registry/default.yaml`)
- [ ] Output: `candidate_id\tbest_target\tsimilarity\tcitation_match\tbib_match`
- [ ] Run across full `styles-legacy/*.csl` corpus; commit top-N report
- [ ] Review top hits manually; file a follow-up bean to bulk-add validated aliases

## Scope
- Generic discovery tool — not T&F-specific
- Targets any parent family (T&F, Elsevier, Springer, Chicago, APA, …)
- Out of scope: structural AST diff (build only if output-diff has too many false negatives)
- Out of scope: auto-patching registry (human review required before alias)

## Verification
- Confirms AAG → taylor-and-francis-chicago-author-date at ≥0.98 similarity
- Runs under 5 minutes across ~2,844 independent styles
- No false positives at ≥0.98 threshold in top 20 hits
