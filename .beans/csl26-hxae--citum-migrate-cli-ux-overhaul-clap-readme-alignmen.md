---
# csl26-hxae
title: 'citum-migrate CLI UX overhaul: clap + README alignment'
status: in-progress
type: task
priority: high
created_at: 2026-05-21T23:13:23Z
updated_at: 2026-05-21T23:13:27Z
parent: csl26-f1u7
---

Replace manual arg parsing with clap derive (fix invisible --help via tracing::debug!), add --version, colored output matching citum CLI. Update README: fix cargo run examples to use installed binary, add missing --emit-evidence/--family-candidate/--minimize-wrapper flags.
