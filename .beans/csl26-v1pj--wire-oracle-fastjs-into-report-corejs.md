---
# csl26-v1pj
title: Wire oracle-fast.js into report-core.js
status: completed
type: task
priority: normal
created_at: 2026-03-06T22:21:07Z
updated_at: 2026-03-06T22:31:35Z
parent: csl26-anlu
---

Route runOracleForStyle() in report-core.js to oracle-fast.js (snapshot) instead of oracle.js (live). Keep oracle.js as regenerator backend only. Add oracleSource field to style report object for compat.html.

## Summary of Changes

- Modified runOracle() in report-core.js: tries oracle-fast.js first, falls back to oracle.js on exit 2 (missing snapshot)
- Added oracleSource field to both native and CSL style report objects
- Added Oracle column to compat.html table (between Format and Dependents)
- Native styles tagged oracleSource: 'citum-native', live fallback tagged 'citeproc-js-live'
