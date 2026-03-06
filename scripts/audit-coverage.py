#!/usr/bin/env python3
"""
Fixture coverage audit for citum-core.

Analyses all reference fixtures and citation fixtures to report:
  - Reference type coverage (what types exist, how many instances)
  - Field coverage matrix per type
  - Contributor shape coverage (single, dual, 3+, 6+, literal, no-author, editor-only, translator)
  - Date shape coverage (year-only, year+month, full, accessed, no-date)
  - Citation scenario coverage
  - Known gaps against the oracle test surface

Usage:
    python scripts/audit-coverage.py [--json] [--fixture path]

Options:
    --json          Output machine-readable JSON instead of text report
    --fixture PATH  Audit a single fixture file instead of all fixtures
"""

import json
import sys
import glob
import os
import argparse
from pathlib import Path
from collections import defaultdict


# ---------------------------------------------------------------------------
# Field groups — fields considered "interesting" for coverage analysis.
# Fields that appear in a type but aren't in this set are still counted
# but not highlighted as gaps.
# ---------------------------------------------------------------------------

# These are the fields the engine/oracle actually renders or tests.
TRACKED_FIELDS = {
    "contributor": ["author", "editor", "translator", "director", "interviewer",
                    "collection-editor", "container-author", "composer", "illustrator",
                    "original-author", "recipient", "reviewed-author", "series-creator"],
    "title": ["title", "container-title", "collection-title", "original-title",
              "reviewed-title", "short-title"],
    "date": ["issued", "accessed", "submitted", "original-date", "event-date"],
    "locator": ["volume", "issue", "page", "edition", "number", "chapter-number",
                "collection-number", "version"],
    "identifier": ["DOI", "ISBN", "ISSN", "PMID", "PMCID", "URL"],
    "publisher": ["publisher", "publisher-place", "archive", "archive-place", "event",
                  "event-place"],
    "descriptor": ["genre", "medium", "note", "abstract", "keyword", "language",
                   "authority", "country", "section"],
}

# Contributor shapes we want to verify are covered
CONTRIBUTOR_SHAPES = {
    "single-author": lambda item: _count_contribs(item, "author") == 1,
    "two-authors": lambda item: _count_contribs(item, "author") == 2,
    "three-authors": lambda item: _count_contribs(item, "author") == 3,
    "six-plus-authors": lambda item: _count_contribs(item, "author") >= 6,
    "corporate-author": lambda item: any(
        "literal" in c for c in item.get("author", [])
    ),
    "no-author": lambda item: "author" not in item or not item["author"],
    "editor-only": lambda item: (
        ("author" not in item or not item["author"])
        and bool(item.get("editor"))
    ),
    "has-translator": lambda item: bool(item.get("translator")),
    "has-editor-and-author": lambda item: (
        bool(item.get("author")) and bool(item.get("editor"))
    ),
}

# Date shapes
DATE_SHAPES = {
    "year-only": lambda item: _date_depth(item, "issued") == 1,
    "year-month": lambda item: _date_depth(item, "issued") == 2,
    "full-date": lambda item: _date_depth(item, "issued") == 3,
    "has-accessed": lambda item: bool(item.get("accessed")),
    "no-date": lambda item: "issued" not in item,
}

# Citation scenarios we expect to cover (checked against citations-expanded.json ids)
EXPECTED_CITATION_SCENARIOS = {
    "single-item",
    "multi-item",
    "with-locator",
    "suppress-author",
    "et-al",
    "prefix",
    "suffix",
    "disambiguate",
    "year-suffix",
    "chapter",
    "report",
    "thesis",
    "webpage",
    "no-date",
}


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _count_contribs(item: dict, role: str) -> int:
    return len(item.get(role, []))


def _date_depth(item: dict, field: str) -> int:
    date = item.get(field)
    if not date:
        return 0
    parts = date.get("date-parts", [[]])
    if not parts or not parts[0]:
        return 0
    return len([p for p in parts[0] if p is not None])


def load_json_fixture(path: str) -> dict | list | None:
    try:
        with open(path) as f:
            return json.load(f)
    except (OSError, json.JSONDecodeError) as e:
        print(f"  WARNING: could not load {path}: {e}", file=sys.stderr)
        return None


def extract_references(data: dict | list) -> dict[str, dict]:
    """Return {id: item} from a fixture, ignoring metadata keys."""
    if isinstance(data, list):
        return {item["id"]: item for item in data if isinstance(item, dict) and "id" in item}
    # dict format — skip string-valued entries like "comment"
    return {
        k: v for k, v in data.items()
        if isinstance(v, dict) and "type" in v
    }


# ---------------------------------------------------------------------------
# Analysis
# ---------------------------------------------------------------------------

def analyse_references(refs: dict[str, dict]) -> dict:
    type_counts: dict[str, int] = defaultdict(int)
    type_fields: dict[str, set] = defaultdict(set)   # type -> set of fields present
    type_missing: dict[str, set] = defaultdict(set)  # type -> fields never seen

    shape_coverage: dict[str, list[str]] = defaultdict(list)  # shape -> [item ids]

    all_fields_seen: set[str] = set()

    for item_id, item in refs.items():
        ref_type = item.get("type", "unknown")
        type_counts[ref_type] += 1

        present = {k for k, v in item.items()
                   if k not in ("id", "class") and v is not None and v != "" and v != []}
        type_fields[ref_type].update(present)
        all_fields_seen.update(present)

        for shape, predicate in CONTRIBUTOR_SHAPES.items():
            if predicate(item):
                shape_coverage[shape].append(item_id)

        for shape, predicate in DATE_SHAPES.items():
            if predicate(item):
                shape_coverage[shape].append(item_id)

    # Identify fields that are tracked but never appear for a given type
    # (only flag for types where the field category is plausibly relevant)
    for ref_type in type_counts:
        seen = type_fields[ref_type]
        for group, fields in TRACKED_FIELDS.items():
            for field in fields:
                if field not in seen and field in all_fields_seen:
                    # Only flag if at least one other type uses this field
                    type_missing[ref_type].add(field)

    return {
        "type_counts": dict(type_counts),
        "type_fields": {t: sorted(f) for t, f in type_fields.items()},
        "type_missing_fields": {t: sorted(f) for t, f in type_missing.items() if f},
        "shape_coverage": {s: ids for s, ids in shape_coverage.items()},
        "total_items": len(refs),
    }


def analyse_citations(citations: list) -> dict:
    scenario_ids = {c["id"] for c in citations if isinstance(c, dict) and "id" in c}
    item_ids_used: set[str] = set()
    for c in citations:
        if isinstance(c, dict):
            for item in c.get("items", []):
                item_ids_used.add(item.get("id", ""))

    matched = EXPECTED_CITATION_SCENARIOS & scenario_ids
    # fuzzy match: scenario name appears as substring of any id
    fuzzy_matched = {
        expected for expected in EXPECTED_CITATION_SCENARIOS
        if any(expected in sid for sid in scenario_ids)
    }
    missing = EXPECTED_CITATION_SCENARIOS - fuzzy_matched

    return {
        "total_scenarios": len(scenario_ids),
        "scenario_ids": sorted(scenario_ids),
        "items_referenced": sorted(item_ids_used),
        "expected_covered": sorted(fuzzy_matched),
        "expected_missing": sorted(missing),
    }


# ---------------------------------------------------------------------------
# Gap analysis
# ---------------------------------------------------------------------------

# Reference types defined in CSL / Citum schema
KNOWN_TYPES = {
    # Serial components
    "article-journal", "article-magazine", "article-newspaper",
    # Collection components
    "chapter", "paper-conference", "entry-encyclopedia", "entry-dictionary",
    "entry-legal",
    # Monographs
    "book", "report", "thesis", "manuscript", "pamphlet", "speech",
    # Legal
    "legal_case", "legislation", "regulation", "treaty", "bill", "hearing",
    # Media
    "broadcast", "motion_picture", "song", "graphic",
    # Technical
    "patent", "standard", "software", "dataset",
    # Personal
    "personal_communication", "post", "post-weblog",
    # Web / other
    "webpage", "interview",
}


def gap_analysis(type_counts: dict[str, int]) -> dict:
    covered = set(type_counts.keys())
    missing = KNOWN_TYPES - covered
    unexpected = covered - KNOWN_TYPES  # types in fixtures not in known set

    return {
        "covered_types": sorted(covered),
        "missing_types": sorted(missing),
        "unexpected_types": sorted(unexpected),  # may be valid Citum extensions
        "coverage_pct": round(100 * len(covered) / len(KNOWN_TYPES), 1),
    }


# ---------------------------------------------------------------------------
# Reporting
# ---------------------------------------------------------------------------

def print_section(title: str) -> None:
    print(f"\n{'=' * 60}")
    print(f"  {title}")
    print(f"{'=' * 60}")


def print_text_report(results: dict, citation_results: dict | None) -> None:
    ref = results["references"]
    gaps = results["gaps"]

    print_section("Reference Type Coverage")
    print(f"  Total items: {ref['total_items']}")
    print(f"  Types covered: {len(ref['type_counts'])} / {len(KNOWN_TYPES)}"
          f"  ({gaps['coverage_pct']}%)")
    print()
    print("  Counts by type:")
    for t, count in sorted(ref["type_counts"].items()):
        print(f"    {t:<30} {count}")

    if gaps["missing_types"]:
        print()
        print("  MISSING types (not in any fixture):")
        for t in gaps["missing_types"]:
            print(f"    - {t}")

    if gaps["unexpected_types"]:
        print()
        print("  Unrecognized types (check spelling / schema extensions):")
        for t in gaps["unexpected_types"]:
            print(f"    ? {t}")

    print_section("Contributor Shape Coverage")
    covered_shapes = [s for s, ids in ref["shape_coverage"].items()
                      if ids and s in CONTRIBUTOR_SHAPES]
    missing_shapes = [s for s in CONTRIBUTOR_SHAPES if not ref["shape_coverage"].get(s)]
    for s in sorted(CONTRIBUTOR_SHAPES.keys()):
        ids = ref["shape_coverage"].get(s, [])
        mark = "OK" if ids else "MISSING"
        print(f"  [{mark:<7}] {s:<30} {', '.join(ids[:3])}{'...' if len(ids) > 3 else ''}")

    print_section("Date Shape Coverage")
    for s in sorted(DATE_SHAPES.keys()):
        ids = ref["shape_coverage"].get(s, [])
        mark = "OK" if ids else "MISSING"
        print(f"  [{mark:<7}] {s:<30} {', '.join(ids[:3])}{'...' if len(ids) > 3 else ''}")

    print_section("Field Coverage Matrix (gaps only)")
    shown = False
    for ref_type, missing in sorted(ref["type_missing_fields"].items()):
        if missing:
            shown = True
            print(f"  {ref_type}:")
            for field in missing:
                print(f"    - {field} (never set for this type, present in other types)")
    if not shown:
        print("  No notable field gaps detected.")

    if citation_results:
        print_section("Citation Scenario Coverage")
        print(f"  Total scenarios in fixture: {citation_results['total_scenarios']}")
        print(f"  Items referenced: {', '.join(citation_results['items_referenced'])}")
        print()
        if citation_results["expected_missing"]:
            print("  MISSING expected scenarios:")
            for s in citation_results["expected_missing"]:
                print(f"    - {s}")
        else:
            print("  All expected citation scenarios covered.")
        print()
        print("  Covered scenarios (sample):")
        for s in citation_results["scenario_ids"][:15]:
            print(f"    - {s}")
        if len(citation_results["scenario_ids"]) > 15:
            print(f"    ... and {len(citation_results['scenario_ids']) - 15} more")

    print_section("Summary")
    n_missing_types = len(gaps["missing_types"])
    n_missing_shapes = sum(
        1 for s in list(CONTRIBUTOR_SHAPES) + list(DATE_SHAPES)
        if not ref["shape_coverage"].get(s)
    )
    n_missing_cite = len(citation_results["expected_missing"]) if citation_results else 0
    print(f"  Missing reference types:       {n_missing_types}")
    print(f"  Missing contributor/date shapes: {n_missing_shapes}")
    print(f"  Missing citation scenarios:    {n_missing_cite}")
    if n_missing_types + n_missing_shapes + n_missing_cite == 0:
        print("\n  All tracked coverage criteria met.")
    else:
        print("\n  See sections above for details on gaps.")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def find_all_reference_fixtures(root: Path) -> list[Path]:
    patterns = [
        "tests/fixtures/references-*.json",
        "tests/fixtures/grouping/*.json",
        "tests/fixtures/multilingual/*.json",
    ]
    paths = []
    for pattern in patterns:
        paths.extend(root.glob(pattern))
    return sorted(paths)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="Output JSON")
    parser.add_argument("--fixture", help="Audit a single fixture file")
    args = parser.parse_args()

    root = Path(__file__).parent.parent  # citum-core root

    # --- Load reference fixtures ---
    all_refs: dict[str, dict] = {}
    fixture_sources: dict[str, str] = {}  # item_id -> fixture path

    if args.fixture:
        paths = [Path(args.fixture)]
    else:
        paths = find_all_reference_fixtures(root)

    if not paths:
        print("ERROR: No reference fixtures found.", file=sys.stderr)
        sys.exit(1)

    for path in paths:
        data = load_json_fixture(str(path))
        if data is None:
            continue
        refs = extract_references(data)
        for item_id, item in refs.items():
            if item_id in all_refs:
                # Cross-domain fixtures (e.g. references-legal and grouping/legal-hierarchy)
                # share IDs intentionally. Only warn if the field data differs.
                existing = all_refs[item_id]
                if existing.get("type") != item.get("type") or existing.get("title") != item.get("title"):
                    print(
                        f"  WARNING: conflicting item id '{item_id}' in {path} "
                        f"(first seen in {fixture_sources[item_id]})",
                        file=sys.stderr,
                    )
                # Keep the richer version (more fields)
                if len(item) > len(existing):
                    all_refs[item_id] = item
                    fixture_sources[item_id] = str(path.relative_to(root))
            else:
                all_refs[item_id] = item
                fixture_sources[item_id] = str(path.relative_to(root))

    # --- Load citation fixture ---
    citation_results = None
    cite_path = root / "tests/fixtures/citations-expanded.json"
    if not args.fixture and cite_path.exists():
        cite_data = load_json_fixture(str(cite_path))
        if isinstance(cite_data, list):
            citation_results = analyse_citations(cite_data)

    # --- Analyse ---
    ref_analysis = analyse_references(all_refs)
    gaps = gap_analysis(ref_analysis["type_counts"])

    results = {
        "references": ref_analysis,
        "gaps": gaps,
        "fixture_sources": fixture_sources,
        "fixtures_loaded": [str(p.relative_to(root)) for p in paths],
    }
    if citation_results:
        results["citations"] = citation_results

    if args.json:
        print(json.dumps(results, indent=2))
    else:
        print_text_report(results, citation_results)


if __name__ == "__main__":
    main()
