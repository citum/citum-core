---
# csl26-rfct
title: Codebase Modularization and Refactor Priorities
status: in-progress
type: milestone
priority: normal
created_at: 2026-05-16T14:30:00Z
updated_at: 2026-05-18T11:22:45Z
---

Analysis of Rust source files (excluding tests) exceeding 800 lines, ranked by refactor priority and grouped by modularization strategy.

## Priority Tiers

### Tier 1: Massive "God" Files (> 2500 lines)
*Urgent modularization required to reduce cognitive load and compilation unit size.*

1. [x] **`citum-schema-style/src/lib.rs` (3607 lines before; facade after this PR)**
   - **Issues:** Orchestrated the entire style model; contained versioning, warnings, and complex template resolution logic.
   - **Target:** Split style-owned schema, validation, inheritance, overlay, section specs, template references, and template resolution into focused modules.
2. [x] **`citum-schema-data/src/reference/mod.rs` (3377 lines before; facade after this PR)**
   - **Issues:** Mixed concerns between core `InputReference` data model and JSON schema generation logic.
   - **Target:** Move `InputReference` and class-discriminator types to specialized modules (`src/reference/input.rs`, `src/reference/classes.rs`) while retaining stable `reference::*` re-exports.
3. [x] **`citum-cli/src/commands.rs` (2923 lines → commands/ module hierarchy)**
   - **Issues:** Command handler monolith.
   - **Target:** Split into `src/commands/*.rs` hierarchy (e.g., `style.rs`, `registry.rs`, `doctor.rs`).

### Tier 2: Large Logic Blocks (1500 - 2500 lines)
*High priority refactoring to separate concerns.*

4. [x] **`citum-schema-style/src/locale/mod.rs` (2410 → 1347 lines, -44%)**
   - **Done:** Extracted embedded en-US data to `src/locale/embedded/en_us.rs` (per original target), the raw → Locale conversion to `src/locale/raw_conversion.rs`, and message-ID mappings + MF2 dispatch to `src/locale/message_ids.rs`. 1306/1306 workspace tests pass.
5. [x] **`citum-schema-data/src/reference/conversion/` (2085 → 767 lines, -63%)**
   - **Done:** Split into `conversion/{legal,scholarly,media}.rs`; shared helpers, RefContext, and the `From<Reference> for InputReference` dispatch stay in `conversion/mod.rs`. Public re-export of `input_reference_from_legacy_edited_book` keeps the existing `citum_schema::reference::conversion::*` path byte-identical. 1306/1306 workspace tests pass.
6. [ ] **`citum-migrate/src/upsampler.rs` (2064 lines)**
   - **Issues:** Complex citation position analysis intertwined with upsampling logic.
   - **Target:** Extract `CitationPositionAnalysis` to `src/upsampler/position.rs`.
7. [ ] **`citum-engine/src/processor/rendering/grouped/core.rs` (1708 lines)**
   - **Issues:** Highly specific rendering logic and classification helpers.
8. [ ] **`citum-io/src/lib.rs` (1622 lines)**
   - **Issues:** Multiple format handlers (BibLaTeX, RIS, JSON).
   - **Target:** Move format-specific logic to `src/formats/`.

### Tier 3: Oversized Modules (1000 - 1500 lines)
*Moderate priority for logical splitting.*

9. **`citum-schema-data/src/reference/types/structural.rs` (1508 lines)**
10. **`citum-schema-style/src/template.rs` (1469 lines)**
11. **`citum-migrate/src/main.rs` (1324 lines)**
12. **`citum-engine/src/processor/disambiguation.rs` (1326 lines)**
13. **`citum_store/src/resolver.rs` (1236 lines)**
14. **`citum-engine/src/values/date.rs` (1212 lines)**
15. **`csl-legacy/src/csl_json.rs` (1185 lines)**
16. **`citum-schema-data/src/citation.rs` (1067 lines)**
17. **`citum-schema-style/src/options/mod.rs` (1063 lines)**
18. **`csl-legacy/src/parser.rs` (1028 lines)**
19. **`citum-cli/src/style_browser.rs` (1000 lines)**

### Tier 4: Long Files (800 - 1000 lines)
*Maintenance and clean-up.*

20. **`citum-schema-style/src/presets.rs` (996 lines)**
21. **`citum-schema-data/src/reference/types/common.rs` (953 lines)**
22. **`citum-schema-style/src/lint.rs` (944 lines)**
23. **`citum-engine/src/render/bibliography.rs` (920 lines)**
24. **`citum-schema-style/src/locale/types.rs` (876 lines)**
25. **`citum-schema-data/src/reference/types/specialized.rs` (801 lines)**

## Strategic Recommendation

Prioritize Tier 1 and Tier 2 refactors, specifically focusing on `citum-schema-style/src/lib.rs` and `citum-cli/src/commands.rs`, as these are the primary entry points and experience the most frequent changes, leading to high friction in PR reviews and local development.
