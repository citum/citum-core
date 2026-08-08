#!/usr/bin/env python3
"""Classify style-variant-builder diff hunks into Citum construct categories.

Prototype for csl26-hv1g: reads the unified diffs that style-variant-builder
uses to derive named CSL variants from its shared chicago-template.csl, and
for each hunk reports which kind of Citum construct it corresponds to (a
citation/bibliography option, a `remove:`/`modify:` type-variant patch, a
macro swap that may already be covered by an axis in
docs/specs/CHICAGO_VARIANT_AXES.md, etc.), auto-generating a YAML fragment
for the categories simple enough to do so unambiguously and flagging the
rest for manual review.

This does not verify rendering correctness. A hunk being classified and
even auto-generated does not mean the resulting Citum YAML renders the same
way citeproc-js does -- that still needs the usual oracle/report-core.js
check.

Usage:
    # Explicit diff paths -- no checkout required.
    python3 scripts/chicago-diff-classifier.py path/to/some.diff [more.diff ...]

    # Default (the three Chicago root diffs) against a local checkout of
    # citation-style-language/style-variant-builder:
    python3 scripts/chicago-diff-classifier.py --builder-dir ../style-variant-builder
    STYLE_VARIANT_BUILDER_DIR=../style-variant-builder python3 scripts/chicago-diff-classifier.py
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

# The three style-variant-builder diffs that Citum's current Chicago root
# styles were migrated from (see docs/specs/CHICAGO_VARIANT_AXES.md).
DEFAULT_ROOT_DIFF_NAMES = [
    "chicago-author-date.diff",
    "chicago-notes.diff",
    "chicago-shortened-notes-bibliography.diff",
]

# Categories the classifier is confident enough about to auto-generate a
# fragment for. Anything not in this set is a "needs review" category by
# default, so a new category added later defaults to being shown in full
# rather than silently summarized.
AUTO_CATEGORIES = {"INFO_METADATA", "ELEMENT_ATTRS", "VARIABLE_REMOVED"}

HUNK_HEADER_RE = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")

INFO_FIELD_RE = re.compile(
    r"<(title|title-short|id|link|summary|updated|category|author|rights)\b"
)
CITATION_BIB_TAG_RE = re.compile(r"^\s*<(citation|bibliography)\b(.*)>")
MACRO_TEXT_RE = re.compile(r'<text\s+macro="([\w-]+)"\s*/>')
COMMENTED_MACRO_RE = re.compile(r'<!--\s*<text\s+macro="([\w-]+)"\s*/>\s*-->')
VARIABLE_TEXT_RE = re.compile(r'<text\s+[^>]*variable="([\w-]+)"[^>]*/>')
SORT_KEY_RE = re.compile(r"<(sort|key)\b")


@dataclass
class Hunk:
    old_start: int
    new_start: int
    removed: list[str] = field(default_factory=list)
    added: list[str] = field(default_factory=list)
    context: list[str] = field(default_factory=list)

    def all_lines(self) -> list[str]:
        return self.context + self.removed + self.added


def parse_hunks(diff_text: str) -> list[Hunk]:
    hunks: list[Hunk] = []
    current: Hunk | None = None
    for line in diff_text.splitlines():
        m = HUNK_HEADER_RE.match(line)
        if m:
            current = Hunk(old_start=int(m.group(1)), new_start=int(m.group(3)))
            hunks.append(current)
            continue
        if current is None:
            continue
        if line.startswith("-") and not line.startswith("---"):
            current.removed.append(line[1:])
        elif line.startswith("+") and not line.startswith("+++"):
            current.added.append(line[1:])
        elif line.startswith(" "):
            current.context.append(line[1:])
    return hunks


def classify(hunk: Hunk) -> tuple[str, str, str | None]:
    """Return (category, human explanation, optional YAML fragment)."""
    if any(INFO_FIELD_RE.search(l) for l in hunk.removed + hunk.added):
        return (
            "INFO_METADATA",
            "Title/id/link/summary/category churn -- handled by ordinary "
            "migration's info: block, not a structural difference.",
            None,
        )

    if any(CITATION_BIB_TAG_RE.match(l) for l in hunk.removed + hunk.added):
        return (
            "ELEMENT_ATTRS",
            "Attribute change on <citation>/<bibliography> -- maps to "
            "citation.options / bibliography.options.",
            attrs_to_yaml(hunk),
        )

    removed_macros = {m.group(1) for l in hunk.removed for m in [MACRO_TEXT_RE.search(l)] if m}
    added_macros = {m.group(1) for l in hunk.added for m in [MACRO_TEXT_RE.search(l)] if m}
    if removed_macros and added_macros and removed_macros != added_macros:
        return (
            "MACRO_SWAP",
            f"Macro reference swap {sorted(removed_macros)} -> "
            f"{sorted(added_macros)}. Check docs/specs/CHICAGO_VARIANT_AXES.md's "
            "axis map first -- several of these (subsequent-note form, "
            "title-primary vs specific-title-first) are already mapped there.",
            None,
        )

    commented = [COMMENTED_MACRO_RE.search(l) for l in hunk.added]
    uncommented_removed = [MACRO_TEXT_RE.search(l) for l in hunk.removed]
    if any(commented) and any(uncommented_removed):
        macro = next(m.group(1) for m in commented if m)
        return (
            "MACRO_SUPPRESSED",
            f"'{macro}' macro call commented out -- a real component "
            "removal/suppression, but the macro itself may render several "
            "variables; inspect the macro body to find the variable(s) to "
            "target.",
            f"# remove or modify+suppress the component(s) rendered by\n"
            f"# macro '{macro}' on the affected reference type(s)",
        )

    removed_vars = {m.group(1) for l in hunk.removed for m in [VARIABLE_TEXT_RE.search(l)] if m}
    added_vars = {m.group(1) for l in hunk.added for m in [VARIABLE_TEXT_RE.search(l)] if m}
    only_removed_vars = removed_vars - added_vars
    if only_removed_vars and not hunk.added:
        return (
            "VARIABLE_REMOVED",
            f"Variable(s) {sorted(only_removed_vars)} dropped outright.",
            "type-variants:\n"
            "  <reference-type>:\n"
            "    remove:\n"
            + "".join(f'      - match: {{variable: {v}}}\n' for v in sorted(only_removed_vars)),
        )

    if any(SORT_KEY_RE.search(l) for l in hunk.removed + hunk.added):
        return (
            "SORT_KEY",
            "Sort key change -- maps to bibliography.options.sort / "
            "citation sort spec, but is style-specific; no safe generic "
            "auto-generation.",
            None,
        )

    return (
        "UNCLASSIFIED",
        "Doesn't match a known pattern -- likely a multi-line macro-body "
        "branch change (a <choose>/<if> restructuring). These are the "
        "hardest category and often correspond to a whole family of "
        "type-variant modify: operations; check CHICAGO_FAMILY_STRATEGY.md's "
        "defect-cluster work for a hand-derived equivalent before "
        "attempting one from scratch.",
        None,
    )


def attrs_to_yaml(hunk: Hunk) -> str | None:
    added_tag = next((l for l in hunk.added if CITATION_BIB_TAG_RE.match(l)), None)
    if not added_tag:
        return None
    attr_re = re.compile(r'([\w-]+)="([^"]*)"')
    attrs = dict(attr_re.findall(added_tag))
    if not attrs:
        return None
    lines = ["options:"]
    for k, v in attrs.items():
        lines.append(f"  {k}: {v}  # from <{'citation' if 'citation' in added_tag else 'bibliography'}> attribute")
    return "\n".join(lines)


def format_hunk_diff(hunk: Hunk) -> list[str]:
    """Render just the changed lines of a hunk, unified-diff style."""
    lines = [f"    - {l}" for l in hunk.removed]
    lines += [f"    + {l}" for l in hunk.added]
    return lines


def report_diff(path: Path) -> dict[str, int]:
    text = path.read_text()
    hunks = parse_hunks(text)
    classified = [(i, hunk, *classify(hunk)) for i, hunk in enumerate(hunks, 1)]

    by_category: dict[str, list] = {}
    for entry in classified:
        by_category.setdefault(entry[2], []).append(entry)
    counts = {cat: len(entries) for cat, entries in by_category.items()}

    review_categories = sorted(
        (c for c in by_category if c not in AUTO_CATEGORIES),
        key=lambda c: -len(by_category[c]),
    )
    auto_categories = sorted(
        (c for c in by_category if c in AUTO_CATEGORIES),
        key=lambda c: -len(by_category[c]),
    )

    print(f"\n{'=' * 70}\n{path.name}  ({len(hunks)} hunks)\n{'=' * 70}")

    if review_categories:
        print("\nNEEDS REVIEW")
        for category in review_categories:
            entries = by_category[category]
            print(f"\n  [{category}] ({len(entries)})")
            for i, hunk, _cat, explanation, yaml_fragment in entries:
                print(f"\n  --- hunk {i} @@ -{hunk.old_start} +{hunk.new_start} @@")
                print(f"      {explanation}")
                for line in format_hunk_diff(hunk):
                    print(f"  {line}")
                if yaml_fragment:
                    print("      Draft Citum fragment:")
                    for line in yaml_fragment.splitlines():
                        print(f"        {line}")

    if auto_categories:
        print("\nAUTO-GENERATABLE (no review needed unless the fragment looks wrong)")
        for category in auto_categories:
            entries = by_category[category]
            print(f"\n  [{category}] ({len(entries)})")
            for i, hunk, _cat, explanation, yaml_fragment in entries:
                print(f"    hunk {i} @@ -{hunk.old_start} +{hunk.new_start} @@ -- {explanation}")
                if yaml_fragment:
                    for line in yaml_fragment.splitlines():
                        print(f"        {line}")

    return counts


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "diffs",
        nargs="*",
        type=Path,
        help="Path(s) to .diff files to classify. If omitted, defaults to "
        f"{', '.join(DEFAULT_ROOT_DIFF_NAMES)} inside --builder-dir.",
    )
    parser.add_argument(
        "--builder-dir",
        type=Path,
        default=None,
        help="Path to a citation-style-language/style-variant-builder "
        "checkout (the directory containing diffs/ and templates/). Only "
        "used when no explicit diff paths are given. Defaults to the "
        "STYLE_VARIANT_BUILDER_DIR environment variable if set.",
    )
    args = parser.parse_args()

    if args.diffs:
        diffs = args.diffs
    else:
        builder_dir = args.builder_dir or os.environ.get("STYLE_VARIANT_BUILDER_DIR")
        if not builder_dir:
            parser.error(
                "no diff paths given and no style-variant-builder checkout "
                "found. Pass explicit .diff paths, --builder-dir <path>, or "
                "set STYLE_VARIANT_BUILDER_DIR."
            )
        diffs = [Path(builder_dir) / "diffs" / name for name in DEFAULT_ROOT_DIFF_NAMES]

    totals: dict[str, int] = {}
    for diff_path in diffs:
        if not diff_path.exists():
            print(f"skip (not found): {diff_path}", file=sys.stderr)
            continue
        counts = report_diff(diff_path)
        for k, v in counts.items():
            totals[k] = totals.get(k, 0) + v

    total_hunks = sum(totals.values())
    print(f"\n{'=' * 70}\nTOTALS across {len(diffs)} diff(s), {total_hunks} hunks\n{'=' * 70}")
    auto = sum(v for k, v in totals.items() if k in AUTO_CATEGORIES)
    for k, v in sorted(totals.items(), key=lambda kv: -kv[1]):
        marker = "(auto-generatable)" if k in AUTO_CATEGORIES else ""
        print(f"  {k:20s} {v:3d}  {marker}")
    print(f"\n  {auto}/{total_hunks} hunks fall in auto-generatable categories; "
          f"{total_hunks - auto} need manual review.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
