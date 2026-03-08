---
# csl26-ul0p
title: Fix conference paper template formatting
status: completed
type: bug
priority: high
created_at: 2026-02-07T06:53:12Z
updated_at: 2026-02-26T14:00:00Z
blocking:
    - csl26-l2hg
---

Conference papers need special formatting with 'in:', 'Presented at', and 'pp.' for page ranges.

Current issues:
- Missing title after editors
- Wrong punctuation around 'pp.'
- 'in:' without space

Fix:
- Extract container prefix ('in:', 'In') from CSL conditionals
- Add page label extraction ('pp.' from CSL Label nodes)
- Handle 'Presented at the [event]' pattern
- Reorder chapter components: 'In:' + editors + title + publisher + pages
- Test against Elsevier Harvard

Expected: In: Ericsson KA, Charness N, ... (eds) The Cambridge Handbook of Expertise. Cambridge University Press, pp 683–703

Refs: GitHub #123, TIER2_PLAN.md Phase 4

## Summary of Changes

Verified resolved via oracle: `elsevier-harvard.csl` passes 32/32 bibliography including the
Ericsson chapter entry (item 4). The chapter template renders `in:` prefix, editor list with
`(Eds.)`, container title, publisher, and `pp.` page range correctly. No code changes required
on this branch — fix was applied in prior migration work.
