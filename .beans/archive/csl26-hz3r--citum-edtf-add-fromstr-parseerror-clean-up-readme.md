---
# csl26-hz3r
title: 'citum-edtf: add FromStr + ParseError, clean up README'
status: completed
type: task
priority: normal
created_at: 2026-06-09T10:19:54Z
updated_at: 2026-06-09T10:26:10Z
---

The public API only exposes winnow-style parse(&mut &str) functions, forcing users to hold mutable bindings and call .unwrap(). Add FromStr for Edtf and Date with a proper ParseError type. Update README to use str::parse() as the primary example.

## Summary of Changes

Added ParseError, FromStr for Edtf and Date. README Usage section rewritten
around str::parse() — no unwrap, no &mut binding. Winnow-style parse/parse_date
functions unchanged. PR #890.
