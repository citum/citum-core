---
# csl26-b4r4
title: Fix flaky Compat Report deploy step
status: completed
type: bug
priority: normal
created_at: 2026-07-06T14:52:15Z
updated_at: 2026-07-06T14:52:42Z
---

Deploy to GitHub Pages step in compat-report.yml intermittently fails with 'Deployment failed, try again later' (transient GitHub Pages API error), requiring manual rerun. Wrap actions/deploy-pages with Wandalen/wretry.action for automatic retry.

## Summary of Changes

Wrapped the `Deploy to GitHub Pages` step in `.github/workflows/compat-report.yml` with `Wandalen/wretry.action@v3.8.0` (pinned SHA `e68c23e6309f2871ca8ae4763e7629b9c258e1ea`), retrying `actions/deploy-pages` up to 3 times with a 30s delay. Confirmed via `gh run list`/`gh run view --log-failed` across the last 100 runs that the Build Docs job always succeeds and only the deploy step's `actions/deploy-pages` call intermittently returns `Deployment failed, try again later.` — a transient GitHub Pages API error, not a build/content problem. Ruled out a concurrent-run race (no other Compat Report or Pages-deploying workflow was in flight at failure time).
