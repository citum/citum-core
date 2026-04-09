#!/usr/bin/env python3
"""Coverage analysis: map CSL variables in test items to Citum schema fields.

Reads a CSL JSON file (or directory of them) and produces a coverage report
showing which CSL variables are used, which map to Citum schema fields, and
which are missing.

Usage:
    python3 scripts/coverage-analysis.py tests/fixtures/test-items-library/chicago-18th.json
    python3 scripts/coverage-analysis.py tests/fixtures/test-items-library/  # all files
    python3 scripts/coverage-analysis.py --json tests/fixtures/test-items-library/chicago-18th.json
"""

import json
import os
import re
import sys
from collections import Counter
from pathlib import Path

# CSL variable regex: starts with a letter, contains letters, numbers,
# hyphens, or underscores. Zotero note-field overrides surface a few
# schema-backed variables with underscores (for example archive_location).
CSL_VAR_PATTERN = re.compile(r"^[A-Za-z][A-Za-z0-9_-]*$")

# CSL variable → Citum schema mapping
# True = present in Citum schema, False = missing, str = notes about mapping
CSL_TO_CITUM = {
    # === Standard variables (present in Citum) ===
    "id": True,
    "type": True,  # mapped to InputReference enum + subtypes
    "title": True,
    "title-short": True,
    "author": True,
    "editor": True,
    "translator": True,
    "interviewer": True,  # Monograph.interviewer
    "recipient": True,  # Monograph.recipient
    "container-title": True,
    "container-title-short": "Serial.short_title",
    "collection-title": "collection_number context (series)",
    "collection-editor": True,  # upsampler handles this
    "collection-number": True,
    "publisher": True,  # Contributor struct with location
    "publisher-place": True,  # Contributor.location
    "issued": True,
    "accessed": True,
    "DOI": True,
    "URL": True,
    "volume": True,
    "issue": True,
    "page": True,
    "number-of-pages": "not in schema (metadata only)",
    "number-of-volumes": "not in schema (metadata only)",
    "edition": True,
    "ISBN": True,
    "ISSN": True,
    "language": True,
    "note": True,
    "abstract": "not rendered; metadata only",
    "genre": True,
    "medium": True,
    "source": "not in Citum (Zotero catalog source)",
    "call-number": "not in Citum (library-local)",
    "citation-key": "mapped to id",
    "shortTitle": "mapped to title-short",
    "journalAbbreviation": "Serial.short_title",
    "archive": True,  # Monograph.archive
    "archive_location": True,  # Monograph.archive_location / ArchiveInfo.location
    "number": True,  # report_number / patent_number / etc.
    "original-date": True,  # Monograph.original_date
    "original-publisher": "Monograph has no original_publisher field",
    "original-publisher-place": "Monograph has no original_publisher_place field",
    "original-title": True,  # Monograph.original_title
    "license": "not in Citum (metadata/provenance only)",
    "script-writer": "not in schema (CSL 1.1 role)",
    "contributor": "not in schema (generic catch-all role)",
    "guest": True,  # Monograph.guest

    # === Variables used via Extra field (note overrides) ===
    # These are set by Zotero via the Extra field because Zotero lacks native UI
    "volume-title": "mapped to container.title for multivolume works",
    "part-number": "mapped to numbering[type=part]",
    "part-title": "mapped to component/part title when present",
    "supplement-number": "mapped to numbering[type=supplement] on serial components",
    "event-title": "mapped to Event.title",
    "event-place": "mapped to Event.location",
    "event-date": "mapped to Event.date",
    "event-location": "alias of event-place",
    "status": "supported on Monograph, SerialComponent, and Event",
    "available-date": "supported on Monograph, SerialComponent, and Event",
    "dimensions": "split between size (physical) and duration (ISO 8601)",
    "references": "supported on Monograph",
    "chapter-number": "mapped to numbering[type=chapter] or legal chapter field",
    "section": "supported on SerialComponent plus legal sources",
    "reviewed-title": "mapped to reviewed.title relation",
    "reviewed-genre": "mapped to reviewed.genre relation",
    "narrator": "mapped to contributors[narrator]",
    "compiler": "mapped to contributors[compiler]",
    "producer": "mapped to contributors[producer]",
    "executive-producer": "mapped to contributors[producer]",
    "host": "mapped to contributors[host]",
    "container-author": "mapped to container.author or reviewed.author, context-dependent",
    "reviewed-author": "mapped to reviewed.author relation",
    "director": "mapped to author for motion_picture",
    "illustrator": "not in schema",
    "composer": "mapped to contributors[composer]",
    "performer": "mapped to contributors[performer]",
    "chair": "not in schema (CSL 1.1 role, rare)",
    "archive_collection": "ArchiveInfo.collection (present in schema)",
    "archive-place": "ArchiveInfo.place (present in schema)",
    "authority": True,  # on LegalCase, Statute, Hearing, Regulation, Brief, Patent, Standard
    "version": True,  # on Dataset, Software
    "scale": "supported on Monograph",
    "submitted": "mapped to issued or filing_date depending on type",
    "original-author": "mapped to original.author relation",
    "PMID": "not in schema (PubMed identifier)",
    "PMCID": "not in schema (PubMed Central identifier)",
}

# Variables that appear in note fields but are Zotero metadata, not CSL
ZOTERO_METADATA_VARS = {
    "OCLC", "IMDb ID", "Distributor", "Page Version ID",
    "Google-Books-ID", "Version Number", "Translated title",
    "Container title", "Reviewed title", "Type", "Section",
}

CANONICAL_CASE_OVERRIDES = {
    "doi": "DOI",
    "url": "URL",
    "isbn": "ISBN",
    "issn": "ISSN",
    "pmid": "PMID",
    "pmcid": "PMCID",
    "shorttitle": "shortTitle",
    "journalabbreviation": "journalAbbreviation",
    "archive_location": "archive_location",
    "archive-collection": "archive_collection",
}

NORMALIZED_METADATA_VARS = {value.lower() for value in ZOTERO_METADATA_VARS}


def canonicalize_variable_name(var_name: str) -> str:
    """Normalize note-field keys to the internal names used by ``CSL_TO_CITUM``."""
    lowered = var_name.strip().lower()
    return CANONICAL_CASE_OVERRIDES.get(lowered, lowered)


def parse_items(filepath: str) -> list[dict]:
    """Load items from a CSL JSON file.

    Accepts either:
    - a top-level array of CSL items; or
    - an object with an "items" key containing the array of items.
    """
    with open(filepath, encoding="utf-8") as f:
        data = json.load(f)

    # Support both common CSL JSON shapes:
    # - top-level array of items
    # - object with an "items" key containing the array
    if isinstance(data, list):
        return data
    if isinstance(data, dict):
        return data.get("items", [])

    # Fallback for unexpected shapes
    return []


def extract_variables(items: list[dict]) -> dict:
    """Extract all CSL variables used across items, including note/Extra overrides."""
    # Standard CSL variables (top-level keys)
    standard_vars: Counter = Counter()
    # Extra-field variables (parsed from note)
    extra_vars: Counter = Counter()
    # Type distribution
    type_dist: Counter = Counter()
    # License/section distribution
    section_dist: Counter = Counter()

    for item in items:
        # Count type
        item_type = item.get("type", "unknown")
        type_dist[item_type] += 1

        # Count standard variables
        for key in item:
            if key in ("id",):
                continue
            if isinstance(item[key], (list, dict)) and item[key]:
                standard_vars[key] += 1
            elif isinstance(item[key], str) and item[key]:
                standard_vars[key] += 1

        # Count license/section references
        lic = item.get("license", "")
        if lic:
            # e.g. "CMOS18 14.127" → "14.127"
            parts = lic.split()
            if len(parts) >= 2:
                section_dist[parts[1].split("(")[0].strip()] += 1

        # Parse Extra field (note) for variable overrides
        note = item.get("note", "")
        if note:
            for line in note.split("\n"):
                line = line.strip()
                if ": " in line or (line.count(":") == 1 and line.endswith(":")):
                    var_name = line.split(":")[0].strip()
                    # Only count as CSL variable if it matches CSL naming style
                    # and is not a known non-CSL Zotero metadata field
                    canonical = canonicalize_variable_name(var_name)
                    if (
                        canonical
                        and CSL_VAR_PATTERN.match(canonical)
                        and canonical.lower() not in NORMALIZED_METADATA_VARS
                    ):
                        extra_vars[canonical] += 1

    return {
        "standard_vars": standard_vars,
        "extra_vars": extra_vars,
        "type_dist": type_dist,
        "section_dist": section_dist,
    }


def classify_coverage(standard_vars: Counter, extra_vars: Counter) -> dict:
    """Classify variables into covered, missing, and partial categories."""
    covered = {}
    missing = {}
    partial = {}
    unmapped = {}

    all_vars = set(standard_vars.keys()) | set(extra_vars.keys())

    for var in sorted(all_vars):
        count = standard_vars.get(var, 0) + extra_vars.get(var, 0)
        is_extra = var in extra_vars and var not in standard_vars

        if var in CSL_TO_CITUM:
            mapping = CSL_TO_CITUM[var]
            if mapping is True:
                covered[var] = {"count": count, "extra_only": is_extra}
            elif mapping is False:
                missing[var] = {"count": count, "extra_only": is_extra}
            else:
                partial[var] = {"count": count, "extra_only": is_extra, "note": mapping}
        else:
            unmapped[var] = {"count": count, "extra_only": is_extra}

    return {
        "covered": covered,
        "missing": missing,
        "partial": partial,
        "unmapped": unmapped,
    }


def print_report(filepath: str, items: list[dict], analysis: dict, classification: dict):
    """Print a human-readable coverage report."""
    print(f"{'=' * 70}")
    print(f"Coverage Analysis: {os.path.basename(filepath)}")
    print(f"{'=' * 70}")
    print(f"Total items: {len(items)}")
    print()

    # Type distribution
    print("Type Distribution:")
    for t, c in analysis["type_dist"].most_common():
        print(f"  {t:30s} {c:4d}")
    print()

    # Section coverage
    sections = analysis["section_dist"]
    if sections:
        print(f"Style sections covered: {len(sections)}")
        print(f"Top 10 sections:")
        for s, c in sections.most_common(10):
            print(f"  {s:10s} {c:3d}")
        print()

    # Coverage classification
    print(f"✅ Covered variables ({len(classification['covered'])}):")
    for var, info in sorted(classification["covered"].items(), key=lambda x: -x[1]["count"]):
        extra = " (Extra only)" if info["extra_only"] else ""
        print(f"  {var:30s} {info['count']:4d}{extra}")
    print()

    print(f"❌ Missing variables ({len(classification['missing'])}):")
    for var, info in sorted(classification["missing"].items(), key=lambda x: -x[1]["count"]):
        extra = " (Extra only)" if info["extra_only"] else ""
        print(f"  {var:30s} {info['count']:4d}{extra}")
    print()

    print(f"⚠️  Partial/noted ({len(classification['partial'])}):")
    for var, info in sorted(classification["partial"].items(), key=lambda x: -x[1]["count"]):
        print(f"  {var:30s} {info['count']:4d}  → {info['note']}")
    print()

    if classification["unmapped"]:
        print(f"❓ Unmapped ({len(classification['unmapped'])}):")
        for var, info in sorted(classification["unmapped"].items(), key=lambda x: -x[1]["count"]):
            print(f"  {var:30s} {info['count']:4d}")
        print()


def json_report(filepath: str, items: list[dict], analysis: dict, classification: dict) -> dict:
    """Create a JSON-serializable report."""
    return {
        "file": os.path.basename(filepath),
        "total_items": len(items),
        "type_distribution": dict(analysis["type_dist"].most_common()),
        "sections_covered": len(analysis["section_dist"]),
        "coverage": {
            "covered": len(classification["covered"]),
            "missing": len(classification["missing"]),
            "partial": len(classification["partial"]),
            "unmapped": len(classification["unmapped"]),
        },
        "missing_variables": {
            var: info for var, info in classification["missing"].items()
        },
        "partial_variables": {
            var: info for var, info in classification["partial"].items()
        },
    }


def main():
    output_json = "--json" in sys.argv
    args = [a for a in sys.argv[1:] if not a.startswith("--")]

    if not args:
        print("Usage: coverage-analysis.py [--json] <file_or_dir>", file=sys.stderr)
        sys.exit(1)

    target = args[0]
    files = []
    if os.path.isdir(target):
        files = sorted(Path(target).glob("*.json"))
    else:
        files = [Path(target)]

    all_reports = []

    for filepath in files:
        items = parse_items(str(filepath))
        if not items:
            continue

        analysis = extract_variables(items)
        classification = classify_coverage(analysis["standard_vars"], analysis["extra_vars"])

        if output_json:
            all_reports.append(json_report(str(filepath), items, analysis, classification))
        else:
            print_report(str(filepath), items, analysis, classification)

    if output_json:
        print(json.dumps(all_reports, indent=2))


if __name__ == "__main__":
    main()
