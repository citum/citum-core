---
# csl26-efsm
title: 'fix(release): repoint STYLE_SCHEMA_VERSION sed paths after schema modularization'
status: todo
type: bug
priority: high
created_at: 2026-05-17T18:34:24Z
updated_at: 2026-05-17T18:34:24Z
---

The schema modularization in commit a905d891 ("refactor(schema): modularize
style schema") moved `pub const STYLE_SCHEMA_VERSION` out of
`crates/citum-schema-style/src/lib.rs` and into a new `src/version.rs`
module. Several release-pipeline call sites still point at the old `lib.rs`
path and will fail silently or noisily when the workflow next runs:

- `.github/workflows/release.yml`
  - line 138: `git add crates/citum-schema-style/src/lib.rs ...`
  - lines 168-169: `sed -n 's/.../\1/p' crates/citum-schema-style/src/lib.rs`
  - lines 248-251: `git diff ... grep -qE '^(crates/citum-schema-style/src/lib.rs|...)'` and the matching sed
- `scripts/bump.py`
  - line 18: `SCHEMA_STYLE_LIB = REPO_ROOT / "crates/citum-schema-style/src/lib.rs"`
  - line 98 error message: "Could not find STYLE_SCHEMA_VERSION in crates/citum-schema-style/src/lib.rs"
- `scripts/test_release_workflow.py`
  - `SCHEMA_LIB` was already repointed at `src/version.rs` in PR #733 as
    the minimal change to unblock CI for the docs work that surfaced this
    bug. The release workflow itself is still broken.

Discovered when PR #733 (docs: integration recipes) added a `scripts/`
file that triggered the `Hygiene Checks — Scripts` job — which had been
skipped on prior PRs by the path filter. The release workflow has been
broken on main since 2026-05-16 ~22:00 UTC; nobody has tried to cut a
release since then.

Fix: replace every `crates/citum-schema-style/src/lib.rs` reference above
with `crates/citum-schema-style/src/version.rs`. Verify by running
`python3 -m unittest scripts.test_release_workflow` and by dry-running
the schema-bump step locally (`python3 scripts/bump.py schema patch
--yes --no-commit --no-tag --no-validate`).
