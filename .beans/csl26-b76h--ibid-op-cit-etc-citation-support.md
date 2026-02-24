---
# csl26-b76h
title: Position-based citation variants (ibid, short-form)
status: done
type: task
priority: high
parent: csl26-5t6s
blocking:
  - csl26-sqsd
created_at: 2026-02-24T07:37:47Z
updated_at: 2026-02-24T13:30:00Z
---

Implements `position` conditions (2,431 uses in corpus; listed as Medium Priority in CLAUDE.md) covering first, subsequent, ibid, and ibid-with-locator. This is a prerequisite for note-style rendering.

Note: op. cit. and loc. cit. are note-style short-form conventions configured at the style level, not processor features. They are out of scope for this task.

See also: `LEGAL_CITATIONS.md`, which defines the `Position` enum (`Ibid`, `IbidWithLocator`, `FarNote`, `ContainerSubsequent`) that this task must align with.

## Implementation Tiers

Three tiers in ascending complexity:

**Tier 1 – Basic position detection (baseline):** Detect `first` vs `subsequent` position within a citation sequence. Required for note styles.

**Tier 2 – Ibid detection:** Detect `ibid` (immediately follows same source) and `ibid-with-locator` (same source, different locator). Aligns with the `Position` enum in `LEGAL_CITATIONS.md`.

**Tier 3 – Page-aware ibid (enhancement only):** When the integration is page-aware (Office JS, LibreOffice UNO), detect "same source, first cite on page." Not a baseline requirement; see Proposed Architectural Options below.

## Context and Constraints
The primary challenge for ibid logic is that citation position can change dynamically as a document is edited. A citation that is ibid on page 5 might move to page 6 and require a full render if it is no longer immediately preceded by the same source.

Page-aware ibid (Tier 3) is an integration enhancement, not a processor requirement. The processor must not be blocked on integration capability.

## Proposed Architectural Options

### Option 1: Context-Rich Processor API
The document-side integration (Word, Typst, etc.) calculates position info (page number, index) and sends it to the processor.
- **Pros:** Processor retains absolute control over formatting and logic.
- **Cons:** High latency; risk of "Layout Loops" where changing string length triggers a page break, which changes the string back, ad infinitum.

### Option 2 (Hybrid): Rendered Fallback with Contextual Overrides
The processor always returns a fully rendered "default" citation but includes a payload of contextual variants that the integration can optionally swap in.

#### Strategy: Graceful Enhancement
1. **The Processor's Role:** It pre-calculates what the citation *would* look like in various states (`ibid`, `ibid-with-locator`, `op-cit`) using the style's rules. It packages these into the metadata.
2. **The Integration's Role:** It renders the default string by default. If the integration is "page-aware" and detects a match (e.g., "same source on same page"), it swaps the default for the provided `ibid` variant.

#### Why this is superior:
- **Universal Compatibility:** Simple integrations (Markdown) just use the default string. They don't break; they just lack advanced features.
- **Layout Stability:** The document-side integration can perform the swap *after* or *during* the layout pass, avoiding circular dependencies between string length and page breaks.
- **Stylistic Integrity:** The processor still generates the strings, so italics, punctuation, and localization are handled by the CSL logic, not hardcoded in the plugin.

#### Conceptual Payload
```json
{
  "rendered": "Smith, 2020, p. 45",
  "contextual_overrides": {
    "ibid": "Ibid.",
    "ibid_with_locator": "Ibid., p. 45"
  }
}
```

## Recommendation
Implement the **Hybrid Model**. It treats positional logic as a "Document-Side UI Concern" while maintaining the CSL processor as the "Source of Truth" for formatting.

## References
[1] Microsoft Word Office JS API (Range.getLocation)
[2] LibreOffice UNO API Layout Information
[3] Citavi Word Add-in (evidence of "first on page" ibid support)
