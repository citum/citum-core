# Date Model Specification

## Status
- **Date**: 2026-04-08
- **Status**: Draft / Evaluation

## Objective
Refine the Citum date model to distinguish between content creation/origination (`created`) and formal publication/issue events (`issued`). This addresses the publication-centric bias of existing citation schemas (like CSL) which poorly represent all major resource types, including archival, unpublished, and born-digital material.

## Background
In many descriptive standards (including archival standards like DACS), the primary date of a record is its creation date. Using `issued` for an unpublished letter, personal manuscript, or dataset is semantically incorrect and forces non-traditional data into a publication-oriented mental model.

## Proposed Model

### Core Date Fields
- `created`: (EDTF) The primary creation or origination date of the content (e.g., when the text was written, the dataset compiled, the software release developed, the recording performed). Applicable to both published and unpublished works.
- `issued`: (EDTF) When the work was formally published or released. For unpublished materials, this field remains empty.
- `available`: (EDTF) When a work first became accessible to its intended or actual audience (e.g., online release preceding formal publication, preprint deposits, or internal reports).
- `accessed`: (EDTF) The date a specific digital instance of a resource (e.g., a URL) was last viewed.

*Naming Note: The field name `issued` is chosen for compatibility with existing citation ecosystems (primarily CSL and Zotero). It denotes the formal release/publication date of a work. Future revisions MAY revisit this name (e.g., `published`) if compatibility with those ecosystems becomes less important.*

### Role-Specific Dates
- `recorded`: For audio-visual works, oral histories, or performances.
- `filed`: For patents, legal filings, or bureaucratic records.
- `revised`: For updated editions or versioned documents.

These role-specific dates supplement `created` and `issued`; they do not replace `created` as the canonical creation date.

## Mapping & Downstream Compatibility

### Citum to CSL
- If `type` is a publication type (book, article, etc.), use `issued` when present; if `issued` is missing, fall back to `created` when mapping to CSL `issued`.
- If `type` is an inherently unpublished type (e.g., personal manuscripts, letters, many archival records, personal communications), map `created` to CSL `issued`.
- This ensures Citum remains the semantically "pure" source while maintaining compatibility with legacy formatters.

## Implementation Plan
1. **Schema Update**: Add `created` and other specific date fields to `WorkCore` and relevant specialized structs.
2. **Logic Refinement**: Update rendering and sorting logic to prioritize `created` when `issued` is missing.
3. **Migration**: Provide a script or automated path to migrate `issued` to `created` for unpublished document types in existing datasets.

## References
- [DACS Chapter 2.4: Date](https://saa-ts-dacs.github.io/dacs/06_part_I/03_chapter_02/04_date.html)
- [CSL Date Specification](https://docs.citationstyles.org/en/stable/specification.html#dates)
