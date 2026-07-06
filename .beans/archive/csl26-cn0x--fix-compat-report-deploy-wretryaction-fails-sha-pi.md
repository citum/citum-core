---
# csl26-cn0x
title: 'Fix compat-report deploy: wretry.action fails SHA-pin policy'
status: completed
type: bug
priority: normal
created_at: 2026-07-06T18:14:22Z
updated_at: 2026-07-06T18:14:59Z
---

PR #1018 wrapped actions/deploy-pages with Wandalen/wretry.action, but wretry.action internally re-invokes itself via a version tag (Wandalen/wretry.action@v3.8.0_js_action) rather than a SHA when wrapping a JS action. Org policy requires all actions pinned to full-length commit SHA, so the workflow run fails with 'action ... is not allowed'. Need a retry mechanism that doesn't depend on wretry.action's internal tag-based self-reference.

## Summary of Changes

Replaced the `Wandalen/wretry.action` wrapper around `actions/deploy-pages` with three sequential, SHA-pinned `actions/deploy-pages` steps (30s sleep between attempts), gated with `continue-on-error`/`if: steps.*.outcome == 'failure'`. The `environment.url` now falls back across `steps.deployment1/2/3.outputs.page_url`. This avoids wretry.action's internal behavior of re-invoking itself via a version tag (`Wandalen/wretry.action@v3.8.0_js_action`) when wrapping a JS action, which the org's "actions must be pinned to a full-length commit SHA" policy rejects (confirmed via the PR #1018 run log: https://github.com/citum/citum-core/actions/runs/28801493973/job/85408025295).
