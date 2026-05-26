# Security Audit - Publish Readiness

Date: 2026-05-12

## Summary

This audit covered the whole workspace, including unpublished crates, because
the CLI, local server, resolver stack, PDF path, and FFI/WASM bindings can
still create user-visible security exposure after publishing.

The PR hardens concrete risks without introducing a new architecture:
dependency policy is now reproducible, remote style fetching is conservative by
default, HTTP server payloads have a fixed cap, FFI input validation is
consistent, and authored style templates have resource limits.

## Findings Addressed

- High: remote resolver accepted plaintext HTTP, redirects, URL credentials,
  fragments, and loopback/private IP literals without a central policy. Added
  `RemoteFetchPolicy`, HTTPS-only defaults, exact host allowlists, redirect
  rejection, URL credential rejection, and a 2 MiB response cap.
- High: git-backed style resolution accepted `git+http://` and did not reject
  unsafe in-repository paths. Git resolution now accepts only `git+https://`
  and safe relative style paths.
- Medium: remote cache writes used ad hoc temporary filenames in one path.
  Resolver and registry cache writes now use atomic temporary files.
- Medium: `citum-server` HTTP mode had no request body cap. It now limits RPC
  requests to 8 MiB and remains loopback-only.
- Medium: C FFI entry points had repeated pointer parsing logic and one
  `.unwrap_or("plain")` fallback that hid invalid format strings. FFI parsing is
  centralized and invalid formats now report an error.
- Medium: authored templates could be deeply nested or extremely large after
  deserialization. Style loading now rejects templates above a maximum nesting
  depth or component count.
- Medium: no reproducible advisory/license/source policy existed in CI. Added
  `cargo-audit`, `cargo-deny`, and Dependabot.

## Residual Risk

- YAML/XML parser recursion is still controlled after deserialization, not
  before allocation. A stricter parser-level depth limiter remains a follow-up
  if serde or roxmltree cannot provide one directly.
- Transitive duplicate `icu`, `getrandom`, and `windows-sys` versions remain
  where they come from Typst, Deno/V8, rustls/ring, tempfile, and platform
  dependencies. `cargo-deny` records them as known audit findings instead of
  forcing brittle transitive pins.
- Three RustSec unmaintained warnings are explicitly ignored in CI and
  `cargo-deny`: `RUSTSEC-2024-0320` (`yaml-rust` through `syntect`/Typst),
  `RUSTSEC-2024-0436` (`paste` through V8, hayagriva, and biblatex), and
  `RUSTSEC-2025-0141` (`bincode` through Deno and `syntect`). The direct
  `gix` vulnerability advisories were fixed by upgrading `gix`.
- `cargo-vet` is intentionally deferred. It records trust decisions, so it
  should be initialized after advisory/license/source policy is stable.
- Fuzz targets are added and buildable, but long-running fuzz campaigns are not
  part of normal PR CI.

## Verification

Required gates:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
cargo audit --deny warnings \
  --ignore RUSTSEC-2024-0320 \
  --ignore RUSTSEC-2024-0436 \
  --ignore RUSTSEC-2025-0141
cargo deny check advisories licenses bans sources
cargo +nightly fuzz build
```

Fallback if `cargo-nextest` is unavailable:

```bash
cargo test
```
