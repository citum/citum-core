# Distributed Registries: A Walkthrough

This guide is a hands-on tour of Citum's distributed-resolver UX. Run every
command as you read; each section is self-contained and starts from a
freshly built `citum` binary in a clean working directory.

The model in two sentences: **citum-core ships a tiny embedded registry of
builtin styles, and everything else lives in registries you opt into**
(institutional registries, the Citum Hub, or your own filesystem). Styles
can be referenced by name, by URL, or by content-addressed CID, and any
inheritance edge can be locked to a specific parent version with
`extends-pin`.

> **What you need.** A built `citum` binary (`cargo build --bin citum`)
> and a terminal in this repo. No network is required for sections 1, 4–6,
> or 8; sections 2, 3, and 7 fetch from the public web.

## Status legend

Each command has a status badge:

- **(P1)** Phase 1 — local-only resolution (file://, builtins).
- **(P2)** Phase 2 — remote fetch via HTTPS / Git, registries config.
- **(P3)** Phase 3 — content addressing, integrity pinning, version checks.

## 1. Inspecting a builtin style

Citum ships an embedded registry. Look up APA 7th end-to-end:

```bash
citum style info apa-7th               # (P1)
```

You'll see the title, aliases, fields, and — new in Phase 3 — the canonical
CID and a paste-ready `extends-pin:` line:

```
ID:       apa-7th
Title:    American Psychological Association 7th edition
Source:   embedded
Aliases:  apa, taylor-and-francis-style-p
Summary:  APA 7th edition
Fields:   psychology, social-science
CID:      bafkreicpx6nc4rll4eahyfid2nbxjjli65tf2vjjed75xtl2ymjtjref44
Pin:      extends-pin: bafkreicpx6nc4rll4eahyfid2nbxjjli65tf2vjjed75xtl2ymjtjref44
```

The CID is deterministic: any two builds of `citum` against the same
embedded YAML produce the same string.

To get the CID alone (e.g. for scripting):

```bash
citum style cid apa-7th                # (P3)
# bafkreicpx6nc4rll4eahyfid2nbxjjli65tf2vjjed75xtl2ymjtjref44
```

## 2. Adding a remote registry

A registry is a YAML file served over HTTPS or Git that lists styles by id
and points at their canonical fetch URL. You opt into a registry per
machine:

```bash
citum registry add https://hub.citum.org/registry/default.yaml \
    --name citum-hub                   # (P2)

citum registry list                    # (P2)
# NAME             URL                                        PRIORITY  CACHED
# citum-hub        https://hub.citum.org/registry/default.yaml      50  fresh
# <embedded>       (built-in)                                        0  —
```

Behind the scenes, `citum registry add` writes the entry to
`~/.config/citum/config.yaml`, fetches the registry index, and caches it
under `~/.cache/citum/registries/`. If the registry is unreachable, the
command fails — no half-configured state.

Refresh on demand:

```bash
citum registry update --all            # (P2)
```

Drop a registry:

```bash
citum registry remove citum-hub        # (P2)
```

## 3. Searching and rendering with a remote style

Once a registry is configured, `citum style search` queries across all
sources:

```bash
citum style search "chicago"           # (P2)
# ID                           SOURCE          DESCRIPTION
# chicago-author-date-18th     citum-hub       Chicago 18th, author-date
# chicago-notes-18th           embedded        Chicago 18th, notes
# ...
```

Render directly against a remote URL — no install step:

```bash
citum render refs \
    -b tests/fixtures/references-expanded.json \
    -s https://hub.citum.org/styles/apa-7th.yaml      # (P2)
```

The first run fetches and caches; subsequent runs are offline-fast. Drop
the cache to force a refetch with `citum registry update`.

To install a style permanently (so it works without HTTP, e.g. on a
laptop offline):

```bash
citum style add chicago-author-date-18th --yes        # (P2)
```

## 4. Pinning a parent style

When you author a child style that extends a parent, you usually want the
*exact* parent at *the moment you tested the child*. `extends-pin:` locks
the inheritance edge to a CID — if the parent's content drifts, your child
fails fast instead of rendering against silently changed semantics.

Generate a paste-ready pin block:

```bash
citum style pin styles/apa-7th.yaml --uri https://hub.citum.org/styles/apa-7th.yaml
# extends: https://hub.citum.org/styles/apa-7th.yaml                             (P3)
# extends-pin: cid:bafkreicpx6nc4rll4eahyfid2nbxjjli65tf2vjjed75xtl2ymjtjref44
```

Paste those two lines at the top of your child YAML:

```yaml
extends: https://hub.citum.org/styles/apa-7th.yaml
extends-pin: cid:bafkreicpx6nc4rll4eahyfid2nbxjjli65tf2vjjed75xtl2ymjtjref44
info:
  title: My APA Variant
options:
  # local overrides ...
```

Render the child. The resolver fetches the parent, re-hashes it, compares
to the pin, and proceeds only if they match. Tamper with the pin (change
one character) and the next render aborts:

```
Error: extends-pin integrity check failed for `https://...`:
  expected bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa,
  got      bafkreicpx6nc4rll4eahyfid2nbxjjli65tf2vjjed75xtl2ymjtjref44
```

## 5. Validating a style end-to-end

Before publishing a style, run the full validator:

```bash
citum style validate path/to/my-journal.yaml          # (P3)
# OK   path/to/my-journal.yaml
# CID  bafkreidlmpu64fjpitcvnmjkr7q5ypqvxdwz67gixmbujc3vshvvlcrebq
# Citum >=0.38.0
```

`validate` parses, runs schema validation, walks every `extends` chain,
verifies any `extends-pin` values, and checks `info.citum-version` against
the running engine. Exit code 0 means publishable; non-zero prints what
broke.

For machine consumption:

```bash
citum style validate my-journal.yaml --format json    # (P3)
```

## 6. Declaring engine compatibility

Add `info.citum-version:` to your style to advertise the minimum engine it
needs:

```yaml
info:
  title: Hypothetical Style With New Features
  citum-version: ">=0.38.0"
```

Anyone running an older engine sees a clean error rather than an opaque
deserialization failure:

```
Error: style `my-journal` requires citum-version `>=0.38.0`;
running engine is `0.35.0`
```

The requirement string follows the `semver` crate's `VersionReq` syntax —
caret ranges (`^0.40`), tildes (`~0.38.2`), explicit versions (`=0.38.0`),
and combinations all work. Omit the field for styles that use only stable,
long-lived features.

Find your engine version with `citum --version`.

## 7. Computing a CID for publication

Registry index files list each style by id, fetch URL, and CID. Compute a
style's CID before adding it to a registry:

```bash
citum style cid path/to/my-journal.yaml               # (P3)
# bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa
```

Add an entry to your registry's `styles:` list:

```yaml
# my-registry.yaml
citum-registry-version: "1"
name: "My Institution"
maintainer: "admin@example.org"
styles:
  - id: my-journal-style
    uri: https://styles.example.org/my-journal.yaml
    cid: bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa
    citum-version: ">=0.38.0"
    description: "House style for Example Journal"
```

Serve the registry over any static HTTP host. Users add it with
`citum registry add https://styles.example.org/my-registry.yaml`.

## 8. Air-gapped operation

For air-gapped or offline environments, omit remote registries from the
config or set very long TTLs:

```yaml
# ~/.config/citum/config.yaml
store:
  format: yaml
registries: []          # no remote sources
```

The resolver chain falls through to `StoreResolver` (your installed user
styles) and then `EmbeddedResolver` (the compiled-in builtins). No HTTP
or Git request is ever attempted.

To pre-populate a fleet of offline machines: install the styles you need
on a connected machine via `citum style add`, then copy
`~/.local/share/citum/styles/` to each target.

## 9. Troubleshooting

| Error                                  | Meaning                                                      | Fix                                                       |
|----------------------------------------|--------------------------------------------------------------|-----------------------------------------------------------|
| `style not found: '<name>'`            | The chain exhausted without finding the style.               | Check spelling; `citum registry list`; `citum style list`. |
| `host '<x>' not in resolver allowlist` | A registry's `allowed_hosts` rejects this URL.               | Configure the registry to allow the host or use a different URL. |
| `network error fetching <uri>: ...`    | DNS, TLS, timeout, or git binary missing.                    | Check connectivity; for git URIs, ensure `git` is on PATH. |
| `extends-pin integrity check failed`   | The fetched parent's content does not match the declared CID. | The parent has changed: update the pin (after reviewing) or revert the parent. |
| `style requires citum-version ...`     | Style declares a minimum engine version newer than yours.    | Upgrade `citum`, or use an older style.                   |
| `inheritance loop detected at base ...` | A cycle in `extends:`.                                       | Break the cycle in one of the offending YAMLs.            |
| `inheritance chain exceeds maximum depth of 5` | Too many `extends` hops.                                     | Flatten the chain or reuse a closer ancestor.             |

## What's next

Citum's distributed-resolver story is intentionally minimal in core: small
embedded registry, opt-in remotes, content-addressed pinning. The Citum
Hub is where most users will get most styles. Institutional publishers can
host their own registries via static HTTP and never push a PR upstream.

For the full design, see [`docs/specs/DISTRIBUTED_RESOLVER.md`](../specs/DISTRIBUTED_RESOLVER.md).
