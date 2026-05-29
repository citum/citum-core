---
# csl26-ryj9
title: 'csl 1.2 versioning thread vs citum strategy: assessment'
status: todo
type: task
created_at: 2026-05-29T01:29:09Z
updated_at: 2026-05-29T01:29:09Z
---

Assessment of the citeproc-rs author's CSL 1.2 versioning / forward-compatibility proposals against Citum's own strategy, and what (if anything) is worth borrowing.

## Source

- Post 5: https://discourse.citationstyles.org/t/csl-1-2-planning/1476/5
- Post 7: https://discourse.citationstyles.org/t/csl-1-2-planning/1476/7

The author thinks in terms of **multiple independent implementations**; Citum is a **single engine + single canonical catalog**, so multi-engine coordination machinery is largely moot here.

## What the thread proposes

- **Post 5 — semver version negotiation.** A style declares `version="1.1"` (= `>=1.1.0, <2.0.0`); the engine knows what it supports and bails if its own version is outside the declared range. No automatic feature disabling — the style author must verify against an old engine. Adds `variant="csl-m"` to replace `1.1mlz1`.
- **Post 7 — Rust-`declare_features!`-style feature flags.** An optional top-level `<features><feature name="xxx"/></features>` block that *requires + enables* individual features. Engines throw if a required feature is unsupported, or if a gated syntax feature is used without declaration. Feature lifecycle: `unstable → accepted` (no longer needs declaration) `→ removed` (errors). Canonical names tied to **tracking issues** so engines agree; grep-able across the style repo; `csl-m` shorthand flips the whole extended set on. The linked `declare_features!` fragment is `rustc`'s `rustc_feature` mechanism (state, version, tracking issue) ported into citeproc-rs.

## Fit assessment

- **Post 5 is behind where Citum already is.** `style.version` + `citum check` (compares against `SchemaVersion::default()`, warns when minor > supported) is post 5's mechanism. Citum already has the parts post 5 only gestures at: the SoftDegrade/Pass/HardFail contract, a unified `CompatibilityWarning` channel, and the post-1.0 pinning grammar (`"latest"`/`"1"`/`"1.2"`/`"1.2.3"`). Post 5 leaves degradation as "author's responsibility, test with an old engine"; Citum productized it. Nothing to borrow.
- **Post 7 mostly solves problems Citum opted out of.** Its three value-props — canonical tracking-issue names *so independent engines agree*, per-feature gating *so engine A ships feature X while engine B hasn't*, and grep-ability *to coordinate across implementations* — are all multi-implementation concerns. With one engine + one catalog, per-feature negotiation is redundant with the schema-version axis, which already carries the load via SoftDegrade.
- **Post 7 partly conflicts with a settled decision.** `docs/architecture/EXTENSIBILITY_STRATEGY.md` is explicit: no intermediate rendering surface, no graduated ladder, no experimental-feature path inside style YAML — rendering behavior goes straight into typed core schema as a `minor` bump. An in-style `<features>` declaration block is exactly the "unstable feature gate / incubation ladder" that doc consciously rejects. Adopting it would reopen a closed architectural question.

## Governing Citum docs

- `docs/specs/FORWARD_COMPATIBILITY.md` — SoftDegrade/Pass/HardFail contract, `CompatibilityWarning` channel.
- `docs/reference/SCHEMA_VERSIONING.md` — two-track (code + `STYLE_SCHEMA_VERSION`) semver, post-1.0 pinning grammar, schema changelog.
- `docs/architecture/EXTENSIBILITY_STRATEGY.md` — rendering→core-schema only; `custom.*` inert; no feature-flag incubation path; no executable code in style YAML.

## Borrowable residue (discipline/tooling, NOT mechanism)

- [ ] **Feature → tracking-issue convention.** Post 7's "canonical name = tracking issue" is good changelog hygiene independent of multi-engine coordination. Citum beans *are* the tracking issues; the schema changelog cites them inconsistently today (some entries reference `csl26-*`, most do not). Make "every additive schema changelog entry names its originating bean" a convention in `SCHEMA_VERSIONING.md` / `ENUM_VOCABULARY_POLICY.md`. Near-free win.
- [ ] **Catalog-wide feature-usage query.** Post 7 values grepping the style repo to see who depends on a feature before changing it. Citum can do this *better* — styles are typed YAML, so a "which styles use field/variant X" query would directly inform the deprecation policy in `ENUM_VOCABULARY_POLICY.md`. Tooling, not a format change. Larger; needs a CLI/tooling design.

The lifecycle *states* (`unstable/accepted/removed`) are interesting as vocabulary, but Citum already encodes the meaningful transition (additive→minor / required→major) in the bump contract, so they are mostly cosmetic.

## License note

Reading the linked MPL fragment is fine. The snippet is a macro invocation; the *idea* (feature lifecycle states, tracking-issue naming) is not copyrightable — only the specific MPL source expression is, and it is itself `rustc`'s pattern (Apache/MIT). Rule: do not paste citeproc-rs MPL source into the tree; re-derive concepts independently. Nothing here needs copying anyway — the borrowable bits are conventions, not code.

## Decision

This bean is a **decision record**. Default recommendation: do not adopt either of the thread's mechanisms (post 5 superseded; post 7 conflicts with extensibility decision). The two checkboxes above are *optional* follow-ups for the user to decide on — leave unchecked until chosen. If neither is wanted, this bean can be completed/scrapped as analysis-only.
