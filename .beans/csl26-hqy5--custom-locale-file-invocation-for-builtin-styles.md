---
# csl26-hqy5
title: Custom locale file invocation for builtin styles
status: todo
type: feature
priority: normal
created_at: 2026-05-16T11:28:28Z
updated_at: 2026-05-16T11:28:28Z
---

## Goal

Make it possible to render with a custom (non-embedded) locale file when the style is a builtin alias. Right now the CLI's `--locale <id>` flag has two paths:

- **File-path style** (`-s styles/embedded/apa-7th.yaml`): the resolver searches a sibling `locales/` directory on disk and reads `<id>.yaml`. Custom locales work.
- **Builtin-alias style** (`-s apa`): the resolver consults `citum_schema::embedded::get_locale_bytes()` only. The locale must be baked into the binary at compile time. Custom locales **don't work**.

This is awkward and was surfaced during the `csl26-v6ok` smoke test: a provisional `eu-ES` locale can be exercised end-to-end only via the path-style form, which is non-obvious.

## Design options (decide before coding)

1. **`--locale-file <path>` flag** — explicit override; bypasses the alias resolver entirely. Smallest blast radius.
2. **Disk-first lookup for builtin styles** — always check `./locales/<id>.yaml` first, then fall back to embedded. Friendlier UX inside a citum checkout; risk: pollutes user-namespace from working directory.
3. **User locale store** — already partly scoped under `csl26-erwz` (User style & locale store). May subsume this bean.

## Todo

- [ ] Decide between (1), (2), and "wait for `csl26-erwz`" — likely a small design note in `docs/specs/`
- [ ] Implement chosen option in `crates/citum-cli/src/style_resolver.rs::load_locale_builtin` (or its caller in `setup_processor`)
- [ ] Update `docs/guides/AUTHORING_LOCALES.md` "Verification" section so the smoke test works for both builtin- and file-path styles
- [ ] Test: render with a non-embedded locale against a builtin-alias style

## Related

- Parent feature: `csl26-v6ok` (surfaced the gap)
- Possibly subsumes / subsumed by: `csl26-erwz` (User style & locale store)
- Relevant code: `crates/citum-cli/src/style_resolver.rs::load_locale_builtin` (line ~229)
