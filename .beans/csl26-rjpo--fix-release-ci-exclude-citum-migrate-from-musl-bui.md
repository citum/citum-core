---
# csl26-rjpo
title: 'Fix release CI: exclude citum-migrate from musl builds + upgrade actions to Node.js 24'
status: in-progress
type: bug
priority: high
created_at: 2026-05-22T13:49:04Z
updated_at: 2026-05-22T13:49:04Z
---

Linux musl build jobs failing (404 for rusty_v8 simdutf prebuilt). Fix: exclude citum-migrate from musl release builds in release-binary.sh + update install.sh to gracefully skip it + upgrade all workflow actions to Node.js 24 versions.
