#!/usr/bin/env python3
"""
Replace explicit bibliography sort templates with SortPreset names.

Matches these canonical patterns (with or without `ascending: true`):
  author-date-title : [author, issued, title] or [author, issued]
  author-title-date : [author, title, issued] or [author, title]
  citation-number   : [citation-number]

Skips files where the sort block contains any extra options
(shorten-names, render-substitutions, etc.).
"""

import re
import sys
from pathlib import Path

STYLES_DIR = Path(__file__).parent.parent / "styles"

# Maps sorted tuple of keys → preset name
# Trailing keys are optional tiebreakers; we resolve by full sequence.
PRESETS = {
    ("author",): "author-date-title",
    ("author", "issued"): "author-date-title",
    ("author", "issued", "title"): "author-date-title",
    ("author", "title"): "author-title-date",
    ("author", "title", "issued"): "author-title-date",
    ("citation-number",): "citation-number",
}

# Regex to match the entire sort block (indented under bibliography/citation)
# Captures the block from `  sort:` through its template lines.
SORT_BLOCK_RE = re.compile(
    r"^( {2}sort:\n"           # "  sort:\n"
    r"(?: {4}.*\n)*)",         # any indented lines that follow
    re.MULTILINE,
)

def extract_keys(block: str) -> list[str] | None:
    """Return ordered key list from a sort block, or None if it has extra options."""
    lines = block.splitlines()
    keys = []
    for line in lines[1:]:  # skip "  sort:"
        stripped = line.strip()
        if not stripped:
            continue
        if stripped == "template:":
            continue
        if stripped.startswith("- key:"):
            keys.append(stripped.removeprefix("- key:").strip())
        elif stripped in ("ascending: true", "ascending: false"):
            # ascending flags are fine — part of SortSpec
            if stripped == "ascending: false":
                return None  # descending not covered by presets
        else:
            # Any other option means we can't collapse to a preset
            return None
    return keys if keys else None


def process_file(path: Path) -> tuple[str, str | None]:
    """Return (status, preset_name). Status: 'replaced', 'skipped', 'no-sort'."""
    text = path.read_text()

    matches = list(SORT_BLOCK_RE.finditer(text))
    if not matches:
        return "no-sort", None

    new_text = text
    offset = 0
    replaced_preset = None

    for m in matches:
        block = m.group(0)
        keys = extract_keys(block)
        if keys is None:
            return "skipped", None

        preset = PRESETS.get(tuple(keys))
        if preset is None:
            return "skipped", None

        replacement = f"  sort: {preset}\n"
        start = m.start() + offset
        end = m.end() + offset
        new_text = new_text[:start] + replacement + new_text[end:]
        offset += len(replacement) - len(block)
        replaced_preset = preset

    if new_text != text:
        path.write_text(new_text)
        return "replaced", replaced_preset

    return "no-change", None


def main():
    files = sorted(STYLES_DIR.glob("*.yaml"))
    replaced, skipped, no_sort = [], [], []

    for f in files:
        status, preset = process_file(f)
        if status == "replaced":
            replaced.append((f.name, preset))
        elif status == "skipped":
            skipped.append(f.name)

    print(f"Replaced ({len(replaced)}):")
    for name, preset in replaced:
        print(f"  {name} → {preset}")

    print(f"\nSkipped ({len(skipped)}) — non-standard sort, review manually:")
    for name in skipped:
        print(f"  {name}")


if __name__ == "__main__":
    main()
