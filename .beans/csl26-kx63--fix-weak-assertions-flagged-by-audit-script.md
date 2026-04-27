---
# csl26-kx63
title: Fix weak assertions flagged by audit script
status: draft
type: epic
priority: normal
created_at: 2026-04-27T20:17:18Z
updated_at: 2026-04-27T20:17:25Z
---

160 findings total from `audit-rust-review-smells.py` as of 2026-04-27.

## Track 1 — contains() assertions (101 high, test code)

Split into three PRs by file cluster:

- [ ] PR A: `crates/citum-engine/src/processor/document/tests.rs` (63 findings)
- [ ] PR B: `crates/citum-engine/src/processor/tests.rs` + `tests/bibliography.rs` remainder (29 findings)
- [ ] PR C: `crates/citum-server/tests/rpc.rs` + `tests/document.rs` + schema tests (11 findings)

## Track 2 — string-allocation smells (59 medium, production code)

Lower urgency. Target files:

- [ ] `crates/citum-migrate/src/debug_output.rs` (8 findings)
- [ ] `crates/citum-server/src/rpc.rs` + `crates/citum-cli/src/main.rs` (10 findings)
- [ ] `crates/citum-engine/src/render/html.rs` + migrate passes + schema renderer (remaining)

## Workflow

```bash
# Check remaining findings in a target file
python3 scripts/audit-rust-review-smells.py --all --json \
  | jq '[.findings[] | select(.path | contains("document/tests"))] | length'
```

Each PR: run the target file's tests to capture actual output, replace contains() with assert_eq!, verify zero findings remain in that file before pushing.
