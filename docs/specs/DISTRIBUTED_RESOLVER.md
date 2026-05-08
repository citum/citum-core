# Distributed Registry and Resolver Architecture Specification

**Status:** Active (Phase 3)
**Date:** 2026-05-08
**Related:** bean csl26-r8d2, `docs/specs/STYLE_REGISTRY.md`,
`docs/guides/DISTRIBUTED_REGISTRIES.md`

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

### Threading `&dyn StyleResolver` — implemented as a two-trait bridge

The schema layer needs a resolver hook for non-`file://` URIs but cannot
depend on `citum_store` directly without creating a cycle. Phase 2 resolved
this without introducing a third crate (the spec's earlier
`citum-resolver-api` proposal):

- **`citum-schema-style` defines its own minimal `StyleResolver` trait** —
  one method, `resolve_style(&self, uri: &str) -> Result<Style,
  ResolutionError>`. This is the trait the schema layer calls through.
- **`citum_store` defines a fuller `StyleResolver` trait** that adds
  `resolve_locale`, plus implements *both* traits on every concrete resolver
  (`StoreResolver`, `EmbeddedResolver`, `RegistryResolver`, `FileResolver`,
  `HttpResolver`, `GitResolver`, `CidResolver`, `ChainResolver`).
- `citum_store` resolvers map their richer `ResolverError` variants to
  schema-layer `ResolutionError` via `resolution_error_from_store_error`,
  preserving the typed `IntegrityFailure` and `VersionMismatch` cases.

The relevant resolution entry points:

```rust
pub fn try_into_resolved(self) -> Result<Self, ResolutionError>;
pub fn try_into_resolved_with(
    self,
    resolver: Option<&dyn StyleResolver>,
) -> Result<Self, ResolutionError>;
fn try_into_resolved_recursive_with_depth(
    self,
    resolver: Option<&dyn StyleResolver>,
    visited: &mut HashSet<String>,
    depth: usize,
) -> Result<Self, ResolutionError>;
```

When `resolver` is `None` and the URI scheme is not `file://`, the engine
returns `ResolutionError::UriResolutionFailed` with a clear message
indicating that a remote resolver is required. `MAX_DEPTH` is 5 hops; the
inheritance chain is loop-detected via `HashSet<String>` of visited keys.

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

#### Core Registry (embedded; small builtin set only)

`registry/default.yaml` is the **embedded default registry** — a small,
versioned index of compiled-in builtin styles plus their aliases. It ships
inside `citum-schema-data` via `include_bytes!` and powers `EmbeddedResolver`
without any network access. As of Phase 3 it lists ~12 builtin styles
(APA 7th, Elsevier Harvard, Chicago Notes 18th, MLA, IEEE, AMA, etc.) and
their well-known aliases; the keys are `kind: base` / `kind: profile`, not
URIs. Styles in this index are guaranteed-resolvable offline.

The full ~150-style catalog (and the long tail of CSL-derived styles) does
**not** live in citum-core. Bulk style distribution is the Hub's
responsibility (`hub.citum.org`); `citum-core`'s embedded registry exists
solely to keep the zero-config user path working without a network call. A
publishing organization that wants to distribute its full collection runs
its own registry — see "Federated Registry Protocol" below.

This split keeps `citum-core` small, makes `EmbeddedResolver` air-gap-safe,
and lets the broader ecosystem grow without bottlenecking on PRs to this
repo.

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
    cache_dir: PathBuf,
}
```

Maintains a local shallow clone in the cache directory
(`<cache_dir>/citum/git/<sha256_of_repo_url>/`). First access runs
a shallow clone (`--depth=1`); subsequent accesses re-use the cache if fresh.

Git operations are handled internally by the `gix` (Gitoxide) library. It
performs a shallow clone and extracts the requested file directly from the
`HEAD` tree. This removes the dependency on an external `git` binary. The
`http` feature (which includes `gix`) is opt-in in `citum_store/Cargo.toml`
to keep WASM and embedded builds lean.

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
`<platform_cache_dir>`:

```
<cache_dir>/
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

Cache location follows the XDG Base Directory specification on all Unix platforms
(including macOS): `XDG_CACHE_HOME` or `~/.cache/citum/`. Windows continues to
use native `%LOCALAPPDATA%\citum\`. This is separate from the data directory
used by `StoreResolver` (`XDG_DATA_HOME` or `~/.local/share/citum/` on Unix).

### Phase 3: Content Addressing and Hub Federation

#### CID Integration

Citum's CID format is **CIDv1, raw codec (`0x55`), SHA-256 hash (`0x12`),
multibase `b` (base32 lowercase)**. Style files are opaque YAML or JSON or
CBOR bytes from a Citum perspective; the raw codec is correct because we do
not parse style content as IPLD. The resulting strings begin with `bafkrei…`
(short for `b` multibase + raw + sha-256). A Citum CID is byte-for-byte
interchangeable with `ipfs add --raw-leaves` output for the same input.

A CID provides an immutable, content-addressed style reference:

```yaml
# Fully immutable reference (URI scheme cid:)
extends: cid:bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa

# Mutable URI with integrity pin (sibling field of extends, not nested)
extends: https://hub.citum.org/styles/apa-7th.yaml
extends-pin: cid:bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa
```

`extends-pin` accepts the bare CID (`bafkrei…`) or the `cid:`-prefixed form;
the resolver canonicalizes both before comparing. Verification re-serializes
the fetched parent to canonical YAML, computes its CID, and aborts with
`ResolutionError::IntegrityFailure { uri, expected, actual }` on mismatch.
For byte-exact verification of the originally-fetched bytes (without a YAML
round-trip), use the lower-level
`citum_store::fetch_and_verify_bytes(http, cid_resolver, uri, expected_cid)`
helper before parsing.

`StyleReference` does *not* gain a third Rust variant. Phase 3 routes `cid:`
URIs through the existing `Uri(String)` variant; a `StyleReference::is_cid()`
helper detects the scheme. This avoids breaking deserialization of every
existing extends value while keeping the schema two-variant.

`CidResolver` wraps an `HttpResolver` and routes `cid:` URIs to IPFS HTTP
gateways. The gateway URL is configurable: default
`https://dweb.link/ipfs/`, override via `StoreConfig.cid_gateway` in
`~/.config/citum/config.yaml`. Cache entries for `cid:` URIs are immutable
and never revalidated.

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

The check runs **after** full deserialization but **before** any inheritance
or template-variant resolution: the resolver parses the style, reads
`info.citum_version`, parses it as a `semver::VersionReq`, and compares
against the engine's `env!("CARGO_PKG_VERSION")`. Incompatibility returns
`ResolutionError::VersionMismatch { uri, required, declared }` rather than
allowing an old engine to silently ignore fields it doesn't understand.
Missing `citum-version` is treated as "any version".

The check applies on three paths:
1. The root style's own `info.citum-version` (in
   `try_into_resolved_recursive_with_depth`, before walking `extends`).
2. Each parent fetched through a resolver (in
   `resolve_style_reference_uri`, after the resolver returns).
3. Each `file://` parent loaded directly without a resolver.

**Forward compatibility:** `Style` does use `#[serde(deny_unknown_fields)]`
at the top level today, so newer-engine-only fields would still fail
deserialization on an older engine. The version check therefore acts as a
fast-fail with a meaningful error rather than a cryptic serde message —
authors can declare `citum-version: ">=0.45.0"` and downstream users get
"upgrade your engine" instead of "unknown field `gendered-roles-mf2`".

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

### Phase 2 (complete)

- [x] Schema-layer `StyleResolver` trait threads through
  `try_into_resolved_with(resolver: Option<&dyn StyleResolver>)`;
  `try_into_resolved` delegates with `None`. (Implemented as a two-trait
  bridge with `citum_store::StyleResolver`; the standalone
  `citum-resolver-api` crate proposed in earlier drafts was not needed.)
- [x] `HttpResolver` resolves `https://` URIs and returns a valid `Style`
- [x] FS cache stores and retrieves entries under `<cache_dir>/citum/styles/http/`
- [x] Stale cache entry is served with a warning when the network is unavailable
- [x] `GitResolver` resolves `git+https://` URIs via shallow clone
- [x] `ChainResolver` constructed from `StoreConfig.registries` in priority order
- [x] `RegistryConfig` parsed from `~/.config/citum/config.yaml`
- [x] `max_depth` cap enforced; deep chains return `UriResolutionFailed`
- [x] `citum registry update [--all | <name>]` CLI command invalidates cache entries

Phase-2-flavored items completed in the Phase 3 commits (rolled forward
because they share the same touch surface):

- [x] `HttpResolver` returns `Denied { uri, reason }` for hosts not in the
  allowlist (was previously a generic `StyleNotFound`).
- [x] `NetworkError { uri, reason }` returned for transport-layer failures
  on both `HttpResolver` and `GitResolver` (was previously generic
  `HttpError` / `GitError`).
- [x] `VersionMismatch { uri, required, declared }` returned when
  `info.citum-version` is incompatible with the running engine.

### Phase 3 (complete)

- [x] `cid:` URI scheme accepted in `extends:` via the existing
  `StyleReference::Uri(String)` variant; `StyleReference::is_cid()` helper
  detects the scheme.
- [x] `CidResolver` resolves CID URIs via a configurable IPFS HTTP gateway
  (default `https://dweb.link/ipfs/`, override `StoreConfig.cid_gateway`).
- [x] `extends-pin` field on `Style` triggers CIDv1 integrity verification
  of the resolved parent.
- [x] `VerifyingResolver<R>` middleware returns `IntegrityFailure` on hash
  mismatch.
- [x] `IntegrityFailure { uri, expected, actual }` exists on both
  `ResolverError` (citum_store) and `ResolutionError` (schema).
- [x] CID cache entries (via `HttpResolver`) are revalidated only on
  explicit `citum registry update`.
- [x] `citum style cid <target>` prints the canonical CIDv1 of a style.
- [x] `citum style pin <target>` prints a paste-ready
  `extends:` + `extends-pin:` block.
- [x] `citum style validate <path>` runs schema + extends + pin + version
  checks end-to-end.
- [x] `citum style info <name>` surfaces CID and `citum-version`.
- [x] User-facing walkthrough at `docs/guides/DISTRIBUTED_REGISTRIES.md`.

### Deferred follow-ups (post Phase 3)

These appeared in earlier draft acceptance lists but are intentionally
deferred and tracked separately:

- `citum-server` wiring. A future bean adds `citum_store` as a dependency
  of `citum-server` (with `http`/`git` features) and constructs a
  `ChainResolver` from the same config path used by the CLI. Until then,
  the server caller is responsible for resolving styles and passing
  pre-resolved YAML into the rendering API.
- `RegistryResolver` index fetching with `citum-version` filter at the
  index layer. Today the version check happens per-style after fetch;
  pushing it into the index probe is a reasonable optimization but not
  required for correctness.
- Locale CID/integrity. Locale resolution does not currently flow through
  the schema-layer chain; remote locale fetching is an open design
  question handled in a separate spec.
- `citum-bindings` WASM CID stubs. The WASM build remains
  `EmbeddedResolver`+`StoreResolver`-only by `#[cfg]`; remote resolution
  belongs server-side.
- Registry discovery (`.well-known/citum-registry.yaml`, Hub
  meta-registry). Forward-compat hooks are documented above; no
  implementation in this round.
- IPFS gateway redundancy / multi-gateway fallback. A single configurable
  gateway is sufficient for v1.

### Surface Exposure

`citum_store` is a library crate. Phase 3 completes the resolver stack; the
resolver is then surfaced through three interfaces:

**CLI (`citum-cli`)** — already wired. `citum render` and `citum registry`
commands construct a `ChainResolver` from `~/.config/citum/config.yaml` and
pass it through the render pipeline. `http` and `git` features are enabled in
`citum-cli/Cargo.toml`.

**WASM binding (`crates/citum-bindings`, feature `wasm`)** — the binding crate
in this repo exposes `renderCitation`, `renderBibliography`, and
`materializeStyle` to JavaScript via `wasm_bindgen`. Remote resolution
(`HttpResolver`, `GitResolver`) is gated out with
`#[cfg(not(target_arch = "wasm32"))]`; the WASM build uses only
`EmbeddedResolver` and `StoreResolver`. Callers (e.g. citum-hub's wasm-bridge)
must resolve remote styles server-side and pass the resolved YAML string into
`materializeStyle` before rendering.

**Server (`crates/citum-server`)** — not yet wired. Phase 3 should add
`citum_store` as a dependency of `citum-server` (with `http`/`git` features
enabled) and construct a `ChainResolver` from the same config path used by the
CLI. The server then resolves styles before passing them to the engine, mirroring
the CLI integration added in Phase 2.

## Changelog

- 2026-05-04: Initial spec (Phases 2 and 3). Phase 1 already implemented.
- 2026-05-08: Phase 3 active. CID content-addressing, `extends-pin`
  integrity verification, `info.citum-version` engine-compat checks,
  typed resolver-error variants (`Denied` / `NetworkError` /
  `VersionMismatch` / `IntegrityFailure`), and the `citum style
  cid` / `pin` / `validate` CLI commands shipped together. Spec
  amendments: dropped the `citum-resolver-api` proposal in favor of
  the two-trait bridge actually shipped in Phase 2, fixed the
  bundled-vs-served core-registry contradiction, specified CIDv1
  raw-codec encoding, clarified `extends-pin` schema location, and
  moved server wiring + locale CID + registry discovery to deferred
  follow-ups.
