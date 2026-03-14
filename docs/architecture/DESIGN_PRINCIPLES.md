# Citum Design Principles

See CLAUDE.md for active behavioral policy. This document captures the full design rationale.

## 1. High-Fidelity Data & Math Support

- **EDTF as Primary**: Prioritize Extended Date/Time Format (EDTF) for all date fields. The engine must support ranges, uncertainty, and approximations natively.
- **Math in Variables**: Support mathematical notation and rich text within metadata variables (e.g., title or note). Prefer standard encodings (e.g., Unicode) over format-specific markup where possible, while ensuring the processor can handle complex fragments without corruption. Ref: csln#64
- **Scoped Multilingualism**: Support multilingual/multiscript data via field 'scopes' (e.g., author+an:mslang). Ref: csln#66
- **Contributor Distinction**: Maintain a strict distinction between individual and organizational authors.

## 2. Hybrid Processing Architecture

- **Dual-Mode Support**: The architecture must cater to both Batch Processing (CLI-based like Pandoc/LaTeX) and Interactive/Real-time usage (GUI-based like Word/Zotero).
- **JSON Server Mode**: Consider a service-oriented approach (similar to Haskell citeproc) where the engine can run as a background process to minimize startup latency for interactive apps.

## 3. Future-Proofing & Versioning (Stability)

- **Forward/Backward Compatibility**: We must ensure that a style written in 2026 works in 2030, and ideally, that a newer style degrades gracefully in an older engine.
- **Schema Evolution**: Utilize Serde's `#[serde(default)]` and `#[serde(flatten)]` to handle unknown or new fields gracefully. Implement a versioning strategy within the Rust types to allow for non-breaking extensions to the specification.

**Strategy: Strict Typing with Explicit Extensions**
1. **Explicit Versioning**: Styles include a `version` field for unambiguous schema identification.
2. **Strict Validation**: Style and locale types use `deny_unknown_fields` to catch typos at parse time. `InputReference` variants are an exception: serde's internally-tagged enum dispatch replays the tag field into the inner struct's deserializer, making `deny_unknown_fields` incompatible. Unknown fields on reference types silently produce `None` and surface as missing data at render time rather than a parse error. This is an accepted trade-off for deterministic class-based dispatch.
3. **Explicit Extension Points**: Styles use explicit `custom: Option<HashMap<String, serde_json::Value>>` fields for user-defined metadata and extensions. See [EXTENSIBILITY_STRATEGY_2026-03-14.md](./EXTENSIBILITY_STRATEGY_2026-03-14.md) for the portability-first extension ladder and limits on executable extensions.
4. **Extension via Defaults**: All new features must be `Option<T>` with `#[serde(default)]`.

**Graceful Degradation for Multilingual Data**
- **Fallback Chain**: Multilingual fields must always implement a `Display` fallback (e.g., `Complex.original` -> `Simple string`).
- **Mode Fallback**: If a style requests a `translated` view but the data only provides `original`, the processor must return `original` rather than failing.

## 4. Multilingual Support

- **Explicit Language/Script Tagging**: All multilingual metadata uses BCP 47 tags with script and optional variant (e.g., "ja-Latn-hepburn" for Hepburn romanization)
- **Graceful Degradation**: Simple string → original → transliteration → translation fallback chain via `Display` trait
- **Surface-Level Disambiguation**: Disambiguation matches displayed written forms (transliterated strings if style shows transliteration), NOT PIDs (ORCID/DOI are for identity, not disambiguation)
- **Declarative Modes**: Styles declare viewing preference (original/transliterated/translated/combined) without procedural logic

## 5. Rust Engineering Standards (Code-as-Schema)

- **Serde-Driven Truth**: We use a Code-First approach. The Rust structs and enums are the source of truth for the schema.
- **Total Stability**: Prohibit the use of `unwrap()` or `unsafe`. Use idiomatic Rust `Result` patterns for all processing logic.

## 6. Explicit Over Magic

**The style language should be explicit; the processor should be dumb.**

If special behavior is needed (e.g., different punctuation for journals vs books), it should be expressed in the style YAML, not hardcoded in the processor.

Bad (magic in processor):
```rust
// Processor has hidden logic for journal articles
if ref_type == "article-journal" {
    separator = ", ";
}
```

Good (explicit in style):
```yaml
# Style explicitly declares type-specific behavior
- title: parent-serial
  overrides:
    article-journal:
      suffix: ","
```

This makes styles portable, testable, and understandable without reading processor code.

## 7. Declarative Templates

Replace CSL 1.0's procedural `<choose>/<if>` with flat templates + type overrides:
```yaml
bibliography:
  template:
    - contributor: author
      form: long
    - date: issued
      form: year
      wrap: parentheses
    - title: primary
    - variable: publisher
      overrides:
        article-journal:
          suppress: true  # Journals don't show publisher
```

## 8. Structured Name Input

Names must be structured (`family`/`given` or `literal`), never parsed from strings. Corporate names can contain commas.

## 9. Oracle Verification

All changes must pass the verification loop:
1. Render with citeproc-js → String A
2. Render with Citum → String B
3. **Pass**: A == B (for supported features)

## 9a. Dual Metrics: Fidelity + SQI

Use two metrics with clear priority:

1. **Fidelity** (oracle match) is the hard gate.
2. **SQI** (Style Quality Index) is a secondary optimization metric.

Policy:
- Never accept an SQI improvement that causes a fidelity regression.
- Use SQI to break ties when multiple implementations have comparable fidelity.
- Prefer SQI improvements that increase fallback robustness and concision without changing rendered output.
- When tradeoffs are unavoidable during iteration, restore fidelity before merge and document temporary SQI/fidelity drift explicitly.

## 10. Well-Commented Code

Code should be self-documenting with clear comments explaining:
- **Why** decisions were made, not just what the code does
- Non-obvious behavior or edge cases
- References to CSL 1.0 spec where relevant
- Known limitations or TODOs
