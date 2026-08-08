---
# csl26-hv1g
title: Diff-driven regeneration of Chicago root styles from style-variant-builder's template+diffs
status: todo
type: task
priority: high
tags:
    - migrate
    - chicago
    - style
    - tooling
created_at: 2026-08-08T11:23:25Z
updated_at: 2026-08-08T11:26:17Z
---

Citum's four embedded Chicago styles were each migrated independently from a flattened, fully-built CSL output file, then their extends: relationships (chicago-18-base.yaml, and each other) were reconstructed by hand afterward -- inferred by comparing the independently-migrated YAML files. style-variant-builder's own diffs are a better source of truth for that structure: each is a delta against the *same* shared templates/chicago-template.csl, so the diff itself already says, unambiguously, what's shared versus variant-specific -- no reconstruction needed.

Concrete evidence this matters, not just a theoretical concern: chicago-shortened-notes-bibliography.diff is a delta directly against the template -- it changes the <citation> macro from the template's own default (citation-notes-full-subsequent-author-title) straight to citation-notes-shortened-author-title, and the <bibliography> layout macro similarly, touching nothing chicago-notes.diff touched. In CSL terms, "shortened notes and bibliography" is a *sibling* of "notes-only," not a descendant -- chicago-notes.diff never modified the <citation> element at all. But Citum's chicago-shortened-notes-bibliography-core.yaml does `extends: chicago-notes-18th`, inheriting notes-18th's entire citation grammar rewrite (processing: note, substitute: editor-translator-short, the citation.ibid/citation.subsequent blocks with their own type-variant patches -- none of which the CSL source for "shortened" ever passed through). That's a plausible concrete explanation for why chicago-shortened-notes-bibliography-core has by far the worst exact-parity in the family (13/473, ~3%, versus 172-173/546 for the other two roots) -- the inheritance shape itself may not match the source material's actual structure, and cluster-by-cluster patching has been fixing symptoms on top of a possibly-wrong foundation.

Idea (Bruce's): a script that reads the diffs directly -- not the flattened output files -- classifies each hunk by shape, and translates it into the corresponding Citum construct, generalizing the manual hunk-by-hunk analysis already done in docs/specs/CHICAGO_VARIANT_AXES.md for a handful of example diffs. Scoped first to just the three diffs behind Citum's existing roots (chicago-author-date.diff, chicago-notes.diff, chicago-shortened-notes-bibliography.diff) against the shared template -- not the full 74-variant corpus, which is a much larger and more heterogeneous problem tracked separately (see CHICAGO_VARIANT_AXES.md's own scoping).

Hunk categories seen across these three diffs (from CHICAGO_VARIANT_AXES.md's analysis plus this bean's own read of the third diff):
- <info> metadata (title, id, links, summary) -- skip, or map to the style's own `source:`/`info:` block, already handled by ordinary migration
- <citation .../> element attribute changes (et-al-min, disambiguate-*, collapse, prefix/suffix) -- map to citation.options
- swap of which macro the <citation>/<bibliography> layout points to -- the interesting case: sometimes this is a real structural difference (author-date grammar vs notes grammar -- not expressible as a patch, needs its own head) and sometimes it's exactly the citation.ibid/citation.subsequent axis already mapped in CHICAGO_VARIANT_AXES.md
- commented-out <text macro=".../> -- remove: op (or modify: + suppress: true) on the matching type-variant
- <sort><key .../></sort> changes -- bibliography.options.sort / citation sort spec changes
- macro-body branch changes inside a <choose> (e.g. chicago-author-date.diff's title-primary / source-date-issued-or-status swaps) -- the hardest category; these often correspond to a whole family of type-variant modify: operations across many reference types, and are where CHICAGO_FAMILY_STRATEGY.md's own defect-cluster work (csl26-h7oc) already has hand-derived Citum equivalents for some of them

Not all hunks are equally automatable. The goal of a first script pass is not full automation -- it's classifying each hunk into one of the categories above, auto-generating the trivial ones (metadata, options, remove/suppress, sort), and clearly flagging the ones that need a human/agent to look at the corresponding cluster work already done. Even partial automation would surface, mechanically, whether chicago-shortened-notes-bibliography-core's extends: relationship should be `chicago-18-base` directly (matching the CSL source) rather than `chicago-notes-18th`.

Whatever comes out of this still needs citeproc-js verification -- structural correctness (the diff says what changed) isn't the same as rendering correctness (Citum's declarative model expressing that change correctly). This regenerates candidate YAML and candidate extends structure; it doesn't replace the report-core.js / oracle.js parity check.

- [x] Confirm the sibling-vs-descendant finding above: read chicago-notes.diff and chicago-shortened-notes-bibliography.diff side by side against the template to verify neither depends on the other -- confirmed mechanically by the classifier run below (neither diff's hunks touch the other's macros). Still open: whether chicago-shortened-notes-bibliography-core.yaml's current parity failures actually correlate with content only reachable via chicago-notes-18th's citation grammar -- that needs the render-level check in the next checklist item, not just the structural one.
- [x] Prototype the hunk classifier against just these three diffs; report the split between auto-generated and flagged-for-review hunks -- scripts/chicago-diff-classifier.py. Result: 30 hunks total (22 author-date, 4 notes, 4 shortened-notes-bibliography); 10/30 auto-generatable (6 info-metadata, 3 element-attrs, 1 variable-removed), 12 macro-swaps needing the axis-map cross-check, 8 unclassified <choose> restructurings needing manual/cluster-work cross-check. Confirms the sibling-vs-descendant finding mechanically: chicago-notes.diff's 4 hunks never touch <citation>/<bibliography> macro references at all (only metadata + et-al attributes), while chicago-shortened-notes-bibliography.diff independently swaps both citation and bibliography macros directly off the template's own defaults -- neither diff depends on the other.
- [ ] If the sibling-relationship finding holds, evaluate re-basing chicago-shortened-notes-bibliography-core.yaml onto chicago-18-base directly instead of chicago-notes-18th, as a bounded experiment, verified against citeproc-js before touching the shipped style
- [ ] Report findings; decide whether to extend the classifier to the fourth relationship (taylor-and-francis-chicago-author-date-core's extends: chicago-author-date-18th, which does match its CSL source's actual `rel="template"` relationship, so is lower priority to re-examine)
