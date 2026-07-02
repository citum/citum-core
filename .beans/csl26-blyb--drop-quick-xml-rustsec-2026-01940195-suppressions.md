---
# csl26-blyb
title: Drop quick-xml RUSTSEC-2026-0194/0195 suppressions when Typst stack updates
status: todo
type: task
created_at: 2026-07-02T18:01:16Z
updated_at: 2026-07-02T18:01:16Z
---

RUSTSEC-2026-0194 (quadratic attribute check) and RUSTSEC-2026-0195 (NsReader memory-exhaustion DoS) are suppressed in .cargo/audit.toml and deny.toml because both quick-xml instances are transitive: citationberg/hayagriva pin 0.38.4 and syntect/plist pin 0.39.4, all reached only through the Typst stack used by citum-pdf. The fix requires quick-xml >=0.41, a breaking bump those upstreams have not shipped. When typst (or citationberg/hayagriva/plist directly) release versions on quick-xml >=0.41, bump the deps and remove both suppressions from BOTH files (they must stay in sync). Renovate vulnerabilityAlerts will not auto-resolve suppressed advisories, so this needs a manual periodic check.
