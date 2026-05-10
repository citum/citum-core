---
# csl26-td2q
title: WASM optimization follow-up fixes
status: completed
type: bug
priority: normal
tags:
    - wasm
    - review
created_at: 2026-05-11T00:04:26Z
updated_at: 2026-05-11T00:06:58Z
---

## Overview

Address the two review blockers from the WASM optimization PR without rewriting history.

## Checklist

- [ ] Make `citum-schema` feature propagation explicit in `citum-bindings`.
- [ ] Align the release profile with the PR's fat-LTO claim.
- [ ] Validate the affected build/test surface.
- [x] Record the outcome and complete the bean.

## Summary of Changes

- Set `citum-schema` to `default-features = false` in `citum-bindings` so the small/full WASM feature split controls both schema and engine dependencies explicitly.
- Switched the workspace release profile from thin LTO to fat LTO so the follow-up matches the optimization goal described in the PR.
- Ran the required workspace checks plus targeted `citum-bindings` feature checks for `small-wasm` and `full-wasm`.
