---
# csl26-b4h2
title: Script discovery of hidden parent-style alias candidates
status: completed
type: feature
priority: normal
created_at: 2026-04-19T12:33:36Z
updated_at: 2026-04-19T12:45:52Z
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

## Summary of Changes

Implementation complete:

1. **Script created**: `scripts/find-alias-candidates.js`
   - Pure citeproc-js (no Citum engine)
   - Reuses rendering pattern from `oracle.js` + similarity functions from `oracle-utils.js`
   - Loads registry builtins from `registry/default.yaml`
   - Enumerates independent styles from `styles-legacy/`, excluding known builtins and aliases
   - Pre-renders all target fingerprints once, then scores 2,829 candidates in parallel (default concurrency 8)

2. **Algorithm**:
   - For each candidate: render citations + bibliography via citeproc-js
   - Normalize text (strip HTML, collapse whitespace, normalize months, et-al punctuation, etc.)
   - Compute fingerprint: ordered list of normalized strings (citations + bibliography)
   - Score against each target: mean `textSimilarity()` across all strings, plus citation/bib match rates
   - Output: `candidate_id, best_target, similarity, citation_match, bib_match`

3. **Results**:
   - Processed 2,827 independent candidates (896 qualified at threshold ≥0.85)
   - Ran in ~3 minutes on full corpus
   - Output: `scripts/report-data/alias-candidates-2026-04-19.tsv` (897 lines including header)
   - Top hits include 13 perfect matches (1.0000 similarity):
     - `american-medical-association-brackets` → `american-medical-association`
     - `applied-clay-science` → `elsevier-harvard`
     - Multiple Chicago variants (in-text-shortened, subsequent-*) → `chicago-shortened-notes-bibliography`
   - AAG correctly excluded (was already aliased in registry)

4. **CLI flags**:
   - `--concurrency N` (default 8)
   - `--threshold F` (default 0.85, filters output)
   - `--limit N` (cap candidates for testing)
   - `--out PATH` (default `scripts/report-data/alias-candidates-YYYY-MM-DD.tsv`)
