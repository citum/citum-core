---
# csl26-drtv
title: Fix dependencies and complete Renovate automation
status: in-progress
type: bug
priority: high
created_at: 2026-11-14T17:01:18Z
updated_at: 2026-11-23T21:12:39Z
---

Patch four affected lockfile packages and repair Renovate configuration and future update coverage.

- [x] Apply targeted security updates
- [x] Repair Renovate warning and enable OSV and daily lockfile maintenance
- [x] Add strict pinned configuration validation in CI
- [x] Validate audit, dependency policy, Rust gate, workflow security, and Renovate dry run
- [x] Prepare review and verify PR CI
- [ ] Verify hosted Renovate configuration after merge

## Validation evidence

- just pre-commit: passed; fmt and clippy clean, 2,838 tests passed.
- cargo audit --deny warnings --no-yanked: passed.
- cargo deny check advisories licenses bans sources: passed with existing policy warnings.
- Renovate 44.239.1 strict repository validation on Node 24.15.0: passed.
- actionlint 1.7.12 and zizmor 1.23.1: passed for the new workflow.
- Read-only extraction found 199 Cargo dependency references across 19 manifests and confirmed daily maintenance, immediate security PRs, Monday routine updates, and automerge disabled.
- Hosted verification remains pending merge.

Authenticated Renovate lookup completed and exercised OSV scanning. OSV reported GHSA-98p4-xjmm-8mfh for gix 0.83.0 after skipping an invalid version event; GitHub confirms only gix <0.62 is affected, so no suppression or extra dependency update was added. See https://github.com/advisories/GHSA-98p4-xjmm-8mfh.

Pre-push policy also exposed the pre-existing invalid Renovate deps commit scope. Cargo, lockfile maintenance, and security updates now inherit the allowed tooling scope; Actions retain ci.

## Security review follow-up

PR #1309 passed all applicable CI checks on ca5496c7, but code scanning reported ref-version-mismatch in the setup-node annotation. The existing SHA is verified against upstream v6.4.0; changing the floating v6 comment to v6.4.0 resolves the mismatch without changing executed code. The follow-up is validated with authenticated online zizmor and actionlint, then PR checks are watched again. Hosted Renovate verification remains pending merge.

## CI installer reliability

The documentation hygiene run failed before linting because the SHA-pinned alint action defaults its binary to latest, which calls an unauthenticated GitHub releases API and received HTTP 403. Pin both alint consumers to binary v0.22.1, matching the earlier successful CI run, so installation skips that lookup. Fold the setup-node annotation correction and installer fix into the original implementation commit as requested.

Pinned alint v0.22.1 installation succeeded with SHA-256 verification and no latest-release API request. Its repository check passed all 29 rules. Actionlint passed all changed workflows. Online zizmor found no issues in the new validator workflow; the two existing workflows have identical baseline findings before and after the version input changes (42 each, zero introduced).


## November 23 RustSec follow-up

- [x] Upgrade gix to 0.91.0 and remove the affected heapless 0.8.0 from the workspace lockfile.
- [x] Update url to 2.5.9 in the workspace and fuzz lockfiles.
- [x] Verify both lockfiles with cargo audit and pass cargo deny.
- [x] Verify the hosted dependency audit on main after merge.

The gix upgrade resolves faster-hex 2.0.0 without its optional heapless
dependency. Remove the existing RUSTSEC-2026-0186 exception from both
audit configurations because the lockfile already contains a patched
memmap2 release.


## Cargo bundle compatibility

- [x] Restore dependency resolution for the Typst, reqwest and Specta upgrades.
- [x] Verify embedded Deno parity with the updated runtime.
- [x] Generate and type-check TypeScript declarations using the public exporter.
- [x] Compile a one-page PDF and verify the retained Typst source.
- [ ] Verify the repaired Cargo bundle in hosted PR checks.

The post-merge dependency audit passed on main at 429d7969 in run
45214241231. The superseded URL update in PR #1318 is closed.
