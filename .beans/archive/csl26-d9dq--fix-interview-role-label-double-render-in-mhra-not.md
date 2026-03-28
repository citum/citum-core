---
# csl26-d9dq
title: Fix interview role-label double-render in mhra-notes-publisher-place-no-url
status: completed
type: bug
priority: normal
created_at: 2026-03-28T01:25:58Z
updated_at: 2026-03-28T10:24:12Z
blocking:
    - csl26-dw4u
---

Citation item 24 (interview) renders 'Stephen Colbert, Stephen Colbert (Interviewer),...' — the interviewer appears twice, the second time with a role label suffix. The identical interview type-variant in mhra-notes-publisher-place (without -no-url) passes 34/34. Root cause is unknown — likely a style-context difference or a file corruption introduced during the Python artifact-removal pass on 2026-03-27. Investigate by diffing the two YAML files and re-running citum render directly for this scenario. Fix is YAML-only.

## Investigation (2026-03-28)

Engine confirmed: `contributor: interviewer form: long name-order: given-first` renders the interviewer name **twice** — once as a plain name and once with `(Interviewer)` role label — comma-joined as if they were two separate items. All three MHRA note styles exhibit this identically:

```
Stephen Colbert, Stephen Colbert (Interviewer), "The Future of Artificial Intelligence",...
```

mhra-notes and mhra-notes-publisher-place pass 34/34 because their CSL oracle (citeproc-js) also has quirky interview templates that match this output. mhra-notes-publisher-place-no-url fails because its CSL oracle renders cleanly (`Stephen Colbert, "title," interview with Yoshua Bengio,...`).

There is no YAML workaround: `form: long` triggers the double-render, `form: verb` emits a role verb not a name. Fix must be in the Rust engine — likely in how `ContributorRole::Interviewer` is rendered when `form: long` and `name-order: given-first` are combined.

## Summary of Changes

- Fixed MHRA notes citation item 24 (et-al-with-locator): added show-with-locator schema field to TemplateNumber, engine change to respect it, and show-with-locator: true on all citation pages components across mhra-notes, mhra-notes-publisher-place, and mhra-notes-publisher-place-no-url.
- Fixed and-others: text engine bug (was using 'et al' instead of locale 'and others').
- All 3 MHRA notes variants now at 18/18 citations, 32/32 bibliography.
