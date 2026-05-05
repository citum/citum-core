# Distributed Registry and Resolver Architecture Specification

**Status:** Draft
**Date:** 2026-05-04
**Related:** bean csl26-r8d2, `docs/specs/STYLE_REGISTRY.md`

## Purpose

Citum's current resolution logic defaults to a centralized model: embedded
builtins plus a local user store. This spec designs Phases 2 and 3 of the
distributed resolver architecture so that institutional publishers, tool
integrators, and the Citum Hub can each operate independent style registries
without requiring central coordination or manual file installation. Phase 1
(file:// URI resolution, `StyleResolver` trait, concrete resolvers, and
`ChainResolver`) is already implemented.

## Scope

**In scope:**
- Phase 2: `HttpResolver` and `GitResolver` implementations, local caching
  layer, per-registry configuration, the deferred `&dyn StyleResolver`
  threading decision
- Phase 3: CID content-addressing, hash-verification middleware, federated
  Hub registry protocol
- Fully resolved design for Trust & Security, Conflict Resolution, Schema &
  Versioning, and Caching Strategy

**Out of scope:**
- Citum Hub web UI and API server implementation (citum-hub concern)
- CSL 1.0 dependent-style bulk registry (citum-hub concern per STYLE_REGISTRY.md)
- Network transport implementation details below the `reqwest` abstraction layer
- Locale remote fetching (deferred follow-up; locale resolution follows the
  same chain pattern)

## Design

### Deferred Decision: Threading `&dyn StyleResolver`

`try_into_resolved_recursive` currently handles `file://` URIs inline
(citum-schema-style/src/lib.rs). To support `https://` and `git+https://` in
Phase 2, a resolver must be accessible at that call site.

**Decision: Thread `Option<&dyn StyleResolver>` as a parameter.**

Rationale: avoids hidden global/thread-local state; `StyleResolver` is already
object-safe (no associated types, no `Sized` bound); the public API gains an
optional parameter with a `None` default, preserving backward compatibility for
callers that only need embedded-or-local resolution.

**New crate: `citum-resolver-api`** (zero network dependencies, only
`thiserror`) — contains `StyleResolver` trait and `ResolverError` enum. This
breaks what would otherwise be a circular dependency:

```
citum-schema-style → citum-resolver-api ← citum_store
```

`citum_store` re-exports the trait and error type for downstream crates.
`citum-engine` and `citum-cli` depend on `citum_store` for concrete
implementations. Signatures use `citum_resolver_api::StyleResolver` directly
(not `citum_store::StyleResolver`) at the schema layer.

**Naming clarification:** The existing private function
`try_into_resolved_recursive` (which walks the `extends` chain) is **renamed**
to `try_into_resolved_with` and gains the `resolver` parameter. It is not a
new wrapper layered on top; it is the same recursive function with a new
signature. The existing public `try_into_resolved` becomes a thin forwarding
shim that calls it with `None`.

Affected signatures:

```rust
// Public shim — unchanged call sites remain valid
pub fn try_into_resolved(self) -> Result<Self, ResolutionError> {
    self.try_into_resolved_with(None, &mut HashSet::new())
}

// Renamed from try_into_resolved_recursive; gains resolver parameter
pub fn try_into_resolved_with(
    self,
    resolver: Option<&dyn citum_resolver_api::StyleResolver>,
    visited: &mut HashSet<String>,
) -> Result<Self, ResolutionError>

// StyleBase::try_resolve_with_visited gains the same resolver parameter
pub(crate) fn try_resolve_with_visited(
    &self,
    resolver: Option<&dyn citum_resolver_api::StyleResolver>,
    visited: &mut HashSet<String>,
) -> Result<Style, ResolutionError>
```

When `resolver` is `None` and the URI scheme is not `file://`, the engine
returns `ResolutionError::UriResolutionFailed` with a clear message indicating
that a remote resolver is required.

### Phase 2: Remote Fetching and Caching

#### Registry Configuration

Registries are declared in the user's Citum configuration file
(`~/.config/citum/config.yaml` on Linux; platform equivalent elsewhere):

```yaml
registries:
  - name: my-institution
    url: https://styles.example.org/citum-registry.yaml
    trusted: true
    priority: 100
    ttl_secs: 3600
  - name: citum-hub
    url: https://hub.citum.org/registry/default.yaml
    trusted: true
    priority: 50
```

`StoreConfig` gains a `registries: Vec<RegistryConfig>` field. The CLI reads
this config and constructs the `ChainResolver` with registries ordered by
`priority` descending, preceded by the local resolvers.

#### Core Registry (Phase 2, HTTP-served)

All ~150 styles in `styles/` are exposed as a **core registry** served via
`HttpResolver`. Embedding them all at build time is impractical at that volume;
instead, `registry/default.yaml` (embedded via `include_bytes!`) lists each
style with its HTTP URI (GitHub raw URL or Hub URL). Only the small index is
embedded — individual style files are fetched and cached on demand.

A typical user interaction:

```bash
citum style search "chicago"       # fetches + caches only the index
citum style add chicago-author-date # fetches + caches that one file
```

Three fetched styles means three HTTP requests, not a clone of the whole repo.

`EmbeddedResolver` retains the current small set of compiled-in builtins for
true zero-network operation (e.g. air-gapped environments). The core registry
sits above it in the chain and requires Phase 2 networking.

As the ecosystem matures, styles may migrate from the core registry to the Hub
registry. This is non-breaking: a Hub entry for the same style ID takes
precedence once the user adds Hub, and the core registry entry remains as the
fallback. No consumer changes are required when a style migrates.

The core registry is intentionally constrained — a CI-verified testbed, not a
general-purpose distribution channel. Styles that do not meet the quality bar
for citum-core belong in the Hub or in institutional registries.

#### Resolver Chain Order (fixed)

The chain encodes the trust hierarchy:

1. `FileResolver` — `file://` URIs (Phase 1, implemented)
2. `StoreResolver` — user-installed styles in platform data dir (Phase 1, implemented)
3. Remote resolvers sorted by `priority` descending (Phase 2)
4. `EmbeddedResolver` — compiled-in builtins only, zero-network fallback (Phase 1, implemented)

A user-installed style always wins over any remote or builtin with the same ID.
There is no merge semantics — first match in chain order wins.

CLI construction:

```rust
let mut resolvers: Vec<Box<dyn StyleResolver>> = vec![
    Box::new(FileResolver),
    Box::new(StoreResolver::new(data_dir, format)),
];
for registry in config.registries.iter().sorted_by_key(|r| Reverse(r.priority)) {
    resolvers.push(Box::new(RegistryResolver::new(registry, cache.clone())));
}
resolvers.push(Box::new(EmbeddedResolver));
let chain = ChainResolver::new(resolvers);
```

#### HttpResolver

```rust
pub struct HttpResolver {
    registry_url: Url,
    cache: Arc<dyn ResolverCache>,
    allowed_hosts: Vec<String>,  // populated from trusted registry url fields
    max_depth: u8,               // extends-chain depth cap, default 5
}
```

Resolution algorithm:
1. Parse the URI. If the host is not in `allowed_hosts`, return
   `ResolverError::Denied { uri, reason: "host not in allowlist" }`.
2. Compute cache key: `sha256(uri)`.
3. On a valid (non-expired) cache hit, return cached style.
4. Fetch via HTTP GET (`Accept: application/yaml, application/json`). On
   network error, serve stale cache entry with a warning if one exists
   (stale-while-revalidate); otherwise return `ResolverError::NetworkError`.
5. Validate schema version compatibility (see Schema & Versioning).
6. Write to cache with freshness metadata.
7. Return deserialized `Style`.

New `ResolverError` variants required for Phase 2:

```rust
#[error("host not in resolver allowlist: {uri} ({reason})")]
Denied { uri: String, reason: String },

#[error("network error fetching {uri}: {reason}")]
NetworkError { uri: String, reason: String },

#[error("schema version mismatch for {uri}: engine requires {required}, style declares {declared}")]
VersionMismatch { uri: String, required: String, declared: String },
```

The `http` feature is opt-in in `citum_store/Cargo.toml` (adds `reqwest` with
the `blocking` feature) to keep WASM and embedded builds lean.

#### GitResolver

URI scheme: `git+https://github.com/org/repo/styles/apa-7th.yaml@main`

```rust
pub struct GitResolver {
    repo_url: Url,
    branch: String,         // default: "main"
    styles_dir: String,     // path within repo, default: "styles/"
    cache: Arc<dyn ResolverCache>,
    allowed_origins: Vec<String>,
}
```

Maintains a local shallow clone in the cache directory
(`<cache_dir>/citum/git/<sha256_of_repo_url>/`). First access runs
`git clone --depth=1 --filter=blob:none --sparse`; subsequent accesses run
`git fetch --depth=1` if the TTL has expired.

Git operations use `std::process::Command` against the system `git` binary. If
`git` is unavailable, returns `ResolverError::NetworkError` with reason "git
binary not found". The `git` feature is opt-in in `citum_store/Cargo.toml`
(no additional library dependencies).

**When to use GitResolver vs HttpResolver.** `GitResolver` is best suited to
institutional registries where a publisher maintains a private collection and
wants atomic versioning across many styles — the whole repo is checked out
together, so styles stay mutually consistent at a given commit. It is *not*
the right tool for the core registry or Hub: cloning a repo just to fetch a
few styles a user browsed is heavyweight. For any public registry where users
select individual styles on demand, `HttpResolver` is the right choice — it
fetches only what is asked for and caches at the file level.

#### Caching Layer

```rust
pub trait ResolverCache: Send + Sync {
    fn get(&self, key: &str) -> Option<CacheEntry>;
    fn put(&self, key: &str, entry: CacheEntry);
    fn invalidate(&self, key: &str);
}

pub struct CacheEntry {
    pub content: Vec<u8>,
    pub fetched_at: SystemTime,
    pub ttl_secs: u64,
    pub content_hash: [u8; 32],  // SHA-256
    pub uri: String,
}
```

Default implementation `FsCache` stores entries under
`<platform_cache_dir>/citum/`:

```
<cache_dir>/citum/
  styles/<sha256_of_uri>/
    content.yaml          # raw fetched bytes
    meta.json             # { uri, fetched_at, ttl_secs, sha256 }
  registries/<sha256_of_registry_url>/
    index.yaml
    meta.json
  git/<sha256_of_repo_url>/   # shallow clone managed by GitResolver
```

Cache entries are keyed by `sha256(uri)` — never by the URI string itself —
to prevent path traversal in the filesystem.

Default TTL: 24 hours. CID entries: `u64::MAX` (immutable, never revalidated).
TTL `0` = always revalidate. Per-registry override via `ttl_secs:` in config.

Cache location uses `dirs::cache_dir()` (separate from `dirs::data_dir()` used
by `StoreResolver`).

### Phase 3: Content Addressing and Hub Federation

#### CID Integration

A CID (Content Identifier, CIDv1 format from the multiformats ecosystem)
provides immutable, content-addressed style references:

```yaml
# Fully immutable reference
extends: cid:bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi

# Mutable URI with integrity pin
extends: https://hub.citum.org/styles/apa-7th.yaml
extends-pin: bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi
```

When `extends-pin` is present the engine verifies SHA-256 of the fetched
content against the CID after stripping the multibase prefix and codec.
Mismatch returns `ResolutionError::IntegrityFailure { uri, expected, actual }`.

`StyleReference` gains a third variant:

```rust
pub enum StyleReference {
    Base(StyleBase),
    Uri(String),
    Cid(String),  // raw CIDv1 string
}
```

`CidResolver` wraps an `HttpResolver` and routes `cid:` URIs to IPFS HTTP
gateways. The gateway URL is configurable; the default is
`https://dweb.link/ipfs/<cid>`. CID cache entries have TTL `u64::MAX`.

#### Hash Verification Middleware

`VerifyingResolver<R: StyleResolver>` wraps any resolver and SHA-256-checks
the raw bytes before deserializing:

```rust
pub struct VerifyingResolver<R: StyleResolver> {
    inner: R,
    expected_hash: Option<[u8; 32]>,  // None = compute-and-store only
}
```

When `expected_hash` is `Some`, a mismatch returns
`ResolverError::IntegrityFailure`. When `None`, the hash is computed and stored
in the cache entry for future auditing without blocking the load.

#### Federated Registry Protocol

A Citum registry is a static YAML (or JSON) file served over HTTP or Git:

```yaml
# citum-registry.yaml
citum-registry-version: "1"
name: "My Institution"
maintainer: "admin@example.org"
styles:
  - id: my-journal-style
    uri: https://styles.example.org/my-journal.yaml
    cid: bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi
    citum-version: ">=0.20.0"
    description: "House style for Example Journal"
```

The format is intentionally static — no server-side execution. `RegistryResolver`
fetches and caches the index file, then resolves style IDs by looking up the
`uri` field and delegating to `HttpResolver` or `CidResolver`.

Citum Hub publishes its registry at `https://hub.citum.org/registry/default.yaml`.
It is not authoritative for `citum-core` builtins — it extends the ecosystem
without replacing `EmbeddedResolver`.

#### Registry Discovery (future)

As the ecosystem grows, users need a way to find registries without already
knowing their URLs. Two complementary mechanisms are anticipated; neither is
required for Phases 2 or 3 and are noted here for forward compatibility.

**Hub as meta-registry.** The Hub registry format can be extended with a
top-level `registries:` list that points to known third-party registries. A
user who has added the Hub gets a curated directory of other registries at no
extra cost. This requires only a schema addition to the existing registry YAML
format and a corresponding step in `RegistryResolver` to fetch and offer to
add listed registries.

**Well-known URL convention.** Institutions that control a web domain can
publish their registry at a fixed, predictable path:
`https://<domain>/.well-known/citum-registry.yaml` (following RFC 8615). A
`citum registry discover <domain>` command would probe that path and offer to
add the registry if found — no central listing required. This is most useful
for known publishers (journals, universities) where a user knows the
organization's domain but not a specific registry URL.

These two mechanisms compose: Hub provides a curated starting point;
well-known lets any institution self-publish without Hub involvement.

## Key Considerations

### Trust and Security

Citum styles are **declarative YAML data structures** deserialized into Rust
structs via Serde. They contain no executable code. The engine never evaluates
style content as code. This significantly constrains the attack surface compared
to plugin systems that load and run arbitrary code.

| Threat | Mitigation |
|--------|-----------|
| Path traversal via `file://` in `extends` | `FileResolver` rejects relative paths; only absolute paths accepted |
| Supply-chain substitution of a trusted style ID | CID pinning in Phase 3; `extends-pin` for mutable URIs |
| Homograph/typosquat registry domain | Explicit host allowlist; user must opt in per registry |
| Infinite `extends` chain (DoS) | `visited` HashSet loop-detection (Phase 1) + `max_depth` cap (Phase 2) |
| Deeply nested YAML parser DoS | `serde_yaml` recursion limit (already applies) |
| Stale pinned CID pointing to broken style | CID is immutable; breakage surfaces as a typed parse error, not silent corruption |

No sandboxing of style content is required because styles carry no behavior.

**Offline environments:** Air-gapped deployments omit remote registries from
config or set `ttl_secs: 18446744073709551615`. The resolver chain falls through
to `StoreResolver` and `EmbeddedResolver` without any network calls.

### Conflict Resolution

When multiple registries provide the same style ID, the chain order determines
the winner — first match wins, no merge semantics:

```
FileResolver > StoreResolver > remotes (by priority desc) > EmbeddedResolver
```

A user-installed style expresses deliberate intent and must not be silently
shadowed by a remote registry update. Institutional registries beat Hub defaults
so publishers can maintain their own canonical variants. Builtins exist only to
ensure zero-config works.

**Version conflicts:** If a remote registry entry declares
`citum-version: ">=0.25.0"` and the current engine is 0.22.0, the entry is
skipped and resolution continues down the chain. If no compatible entry is
found, `ResolverError::VersionMismatch` is returned.

### Schema and Versioning

`StyleInfo` gains an optional field:

```rust
/// Minimum Citum engine version required to use this style.
#[serde(rename = "citum-version")]
pub citum_version: Option<String>,
```

The engine performs partial deserialization to extract `info.citum-version`
before full deserialization, compares against `env!("CARGO_PKG_VERSION")` via
semver range check, and returns `ResolverError::VersionMismatch` on
incompatibility rather than a cryptic Serde error.

**Forward compatibility:** `#[serde(deny_unknown_fields)]` is NOT used on
`Style`. Unknown fields in remote styles from newer engine versions are
silently ignored, allowing newer styles to load on older engines with
unrecognized features absent.

### Caching Strategy

**Offline-first.** The cache is the source of truth during normal operation.
The network is consulted only when the cached entry is absent or stale, or on
explicit `citum registry update`.

Freshness rules:

| Condition | Behavior |
|-----------|---------|
| Cache hit, not stale | Serve from cache; no network call |
| Cache hit, stale; network succeeds | Fetch, update cache, serve fresh |
| Cache hit, stale; network fails | Serve stale entry with logged warning |
| Cache miss; network succeeds | Fetch, populate cache, serve |
| Cache miss; network fails | Return `ResolverError::NetworkError` |
| CID entry (any age) | Serve from cache; never revalidate |

CLI command `citum registry update [--all | <name>]` deletes the `meta.json`
sidecar for the named registry's entries, forcing revalidation on next access.
There is no automatic background revalidation — the process model is single-shot
CLI invocations.

## User Experience

### End-User CLI Workflows

All registry and style management lives under two namespaces:
`citum registry` and `citum style`.

#### Discovering and adding a registry

```bash
# Add a registry by URL; --name is optional (defaults to hostname)
citum registry add https://styles.example.org/citum-registry.yaml \
    --name my-institution

# List configured registries and their status
citum registry list
# NAME             URL                                        PRIORITY  TTL    CACHED
# my-institution   https://styles.example.org/...             100       1h     2h ago
# citum-hub        https://hub.citum.org/registry/...          50       24h    4h ago
# <embedded>       (built-in)                                   0        —      —

# Refresh cached registry indexes
citum registry update --all
citum registry update my-institution

# Remove a registry
citum registry remove my-institution
```

On `add`, the CLI fetches and validates the registry index, reports how many
styles it found, and writes the entry to `~/.config/citum/config.yaml`. If the
registry is unreachable, the command fails with a clear error — it does not
silently add an invalid entry.

#### Browsing and installing styles

```bash
# Search across all registries
citum style search "chicago"
# ID                           REGISTRY        DESCRIPTION
# chicago-author-date-18th     citum-hub       Chicago 18th ed., author-date
# my-chicago-variant           my-institution  Variant for Example Journal

# List all styles available from a specific registry
citum style list --registry citum-hub

# Show details for a specific style
citum style info chicago-author-date-18th
# ID:          chicago-author-date-18th
# Registry:    citum-hub
# URI:         https://hub.citum.org/styles/chicago-author-date-18th.yaml
# CID:         bafybei...
# Citum:       >=0.20.0
# Description: Chicago 18th edition author-date format

# Install a style into the user store (makes it available offline)
citum style add chicago-author-date-18th
citum style add https://styles.example.org/my-journal.yaml

# Remove a user-installed style
citum style remove chicago-author-date-18th
```

`citum style add` resolves through the chain, downloads the style, and places
it in `<data_dir>/citum/styles/`. Subsequent renders use it from `StoreResolver`
without a network call.

#### Using a style directly by URI (no install required)

```bash
citum render refs \
    -b references.json \
    -s https://hub.citum.org/styles/apa-7th.yaml
```

The CLI passes the URI through the `ChainResolver`. The style is fetched,
cached, and used in one step. No permanent installation occurs.

### Style Authoring Workflows

#### Getting a stable style reference

A style author who wants their style to extend a parent needs a stable
reference. The `citum style pin` command produces one:

```bash
citum style pin apa-7th
# extends: https://hub.citum.org/styles/apa-7th.yaml
# extends-pin: bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi

# Or for a local file
citum style pin file:///path/to/parent.yaml
# extends: file:///path/to/parent.yaml
# extends-pin: bafybei...
```

The output is ready to paste directly into the `extends:` block of the child
style. The `extends-pin` value locks the dependency to the exact content of
the parent at that moment; the engine will reject any fetched version whose
SHA-256 does not match.

#### Declaring engine compatibility

Authors add a `citum-version` field to `info:` to signal the minimum engine
version their style requires:

```yaml
info:
  title: My Journal Style
  citum-version: ">=0.22.0"
```

Use `citum --version` to find the current engine version. The semver range
format follows the `semver` crate's `VersionReq` syntax. Omitting the field
means "any version" — appropriate for styles that use only stable, long-lived
features.

#### Verifying a style before publishing

```bash
# Validate YAML schema and resolve all extends chains
citum style validate my-journal.yaml

# Render test output against the standard fixture set
citum render refs -b tests/fixtures/references.json -s my-journal.yaml

# Compute and display the CID (for inclusion in a registry index)
citum style cid my-journal.yaml
# bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi
```

#### Publishing to a registry

A registry is a static YAML file — authors publish by adding an entry to
their registry's `styles:` list, including the style's URI and CID, and
serving the updated file. No central submission process is required. The Hub
registry (`hub.citum.org`) accepts community submissions via pull request
to its source repository (citum-hub concern, out of scope here).

## Implementation Notes

### WASM Compatibility

`HttpResolver` and `GitResolver` are gated behind
`#[cfg(not(target_arch = "wasm32"))]`. In the WASM build (citum-hub), remote
resolution is handled by the hub server layer, not the WASM module. The WASM
build uses only `EmbeddedResolver` and `StoreResolver`.

### Cargo Feature Flags

`citum_store` gains optional features:
- `http`: adds `reqwest` (blocking, no Tokio runtime required for CLI)
- `git`: no additional library deps (uses system `git` via `Command`)

Both are off by default. `citum-cli` enables both. WASM bridge enables neither.

## Acceptance Criteria

### Phase 2

- [ ] `citum-resolver-api` crate exists; `StyleResolver` and `ResolverError`
  defined there; `citum_store` re-exports both
- [ ] `try_into_resolved_with(resolver: Option<&dyn StyleResolver>, visited)`
  is the canonical recursive entry point; `try_into_resolved` delegates with `None`
- [ ] `HttpResolver` resolves `https://` URIs and returns a valid `Style`
- [ ] `HttpResolver` returns `Denied` for hosts not in the allowlist
- [ ] `FsCache` stores and retrieves entries under `<cache_dir>/citum/`
- [ ] Stale cache entry is served with a warning when the network is unavailable
- [ ] `GitResolver` resolves `git+https://` URIs via shallow clone
- [ ] `ChainResolver` constructed from `StoreConfig.registries` in priority order
- [ ] `RegistryConfig` parsed from `~/.config/citum/config.yaml`
- [ ] `max_depth` cap enforced; deep chains return `UriResolutionFailed`
- [ ] `VersionMismatch` returned when `citum-version` is incompatible with the
  running engine
- [ ] `citum registry update [--all | <name>]` CLI command invalidates cache entries
- [ ] `EmbeddedResolver` serves all styles in `styles/` via the expanded
  `registry/default.yaml`, not only the previous short builtin slice

### Phase 3

- [ ] `StyleReference::Cid` variant accepted in YAML as `cid:<cidv1>`
- [ ] `CidResolver` resolves CID URIs via configurable IPFS HTTP gateway
- [ ] `extends-pin` field triggers SHA-256 integrity verification after fetch
- [ ] `VerifyingResolver` returns `IntegrityFailure` on hash mismatch
- [ ] `RegistryResolver` fetches and caches a registry index; resolves style IDs
- [ ] CID cache entries have TTL `u64::MAX` and are never revalidated

## Changelog

- 2026-05-04: Initial spec (Phases 2 and 3). Phase 1 already implemented.
