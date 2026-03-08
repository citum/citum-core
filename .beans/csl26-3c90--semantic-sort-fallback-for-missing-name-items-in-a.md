---
# csl26-3c90
title: Semantic sort fallback for missing-name items in alphabetical bibliographies
status: todo
type: bug
priority: high
created_at: 2026-03-08T00:33:31Z
updated_at: 2026-03-08T00:33:31Z
---

Recent sorting changes now fall back to title-based behavior for works that have
no author-like contributor in alphabetical bibliographies. That prevents
unnamed works from clustering incorrectly, but the fallback still needs a
style-level review across affected alphabetical families to confirm it matches
publisher intent rather than just citeproc legacy behavior.

Focus on styles such as `springer-basic-brackets-no-et-al-alphabetical` and any
other alphabetical numeric variants that depend on missing-name ordering. Verify
both bibliography order and the surrounding punctuation/layout output against
the authority oracle, then decide whether any remaining mismatches are genuine
style defects or intentional divergences worth documenting.
