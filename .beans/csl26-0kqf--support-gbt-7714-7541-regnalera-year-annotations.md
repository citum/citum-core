---
# csl26-0kqf
title: Support GB/T 7714 §7.5.4.1 regnal/era-year annotations
status: draft
type: feature
priority: normal
tags:
    - multilingual
    - migrate
    - style
    - dates
created_at: 2026-07-18T14:15:31Z
updated_at: 2026-07-18T14:15:55Z
---

GB/T 7714—2025 §7.5.4.1 requires historical citations to show the Gregorian year annotated with the original calendar's year in parens, e.g. 1705（康熙四十四年） (Kangxi reign year 44) or 1947（民国三十六年） (Minguo year 36). Raised by reviewer @YDX-2147483647 on PR #1067 as more common than the copyright/printing/estimated cases fixed there. Existing EraLabels/EDTF-historical-era support (docs/specs/EDTF_HISTORICAL_ERA_RENDERING.md, EDTF_ERA_LABEL_PROFILES.md) is a fixed BC/AD/BCE/CE suffix on negative years only and does not cover this — it is a genuinely different, larger feature requiring calendar-conversion lookup tables.

## Scope notes

- **Minguo (Republic-era) case** — cheap: `year_name = Gregorian - 1911`, no lookup table. Could be a low-cost first slice if ever prioritized.
- **Imperial reign-era case** (e.g. 康熙/Kangxi) — expensive: needs a dynasty/era-name → Gregorian-span lookup table (irregular per-emperor/per-era boundaries, not a formula), Han-numeral ordinal-year formatting, and a new parenthetical dual-year render shape (`format_display_year` in `crates/citum-engine/src/values/date.rs` currently only appends a single trailing suffix string, not a bracketed secondary annotation).
- Needs a data-model decision too: where does the original-calendar year come from on input? Not present in current `InputReference`/EDTF model at all.
- Reference: https://en.wikipedia.org/wiki/Regnal_year#Sinosphere_era_names
- Not attempted in PR #1067 — see `docs/reference/GBT_7714_CITATION_CONVENTIONS.md`.
