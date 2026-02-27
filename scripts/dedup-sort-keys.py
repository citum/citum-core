#!/usr/bin/env python3
"""
Deduplicate sort keys in bibliography sort templates.
Reads each affected style file, deduplicates keys in the sort.template block
(first occurrence wins), and writes back in-place.
"""

from pathlib import Path

# The 16 affected styles from csl26-1mkv
AFFECTED_STYLES = [
    "annual-reviews-author-date.yaml",
    "begell-house-chicago-author-date.yaml",
    "elsevier-vancouver-author-date.yaml",
    "mhra-author-date-publisher-place.yaml",
    "mhra-notes.yaml",
    "mhra-shortened-notes-publisher-place.yaml",
    "modern-language-association.yaml",
    "museum-national-dhistoire-naturelle.yaml",
    "new-harts-rules-author-date-space-publisher.yaml",
    "oscola-no-ibid.yaml",
    "oscola.yaml",
    "pensoft-journals.yaml",
    "sage-harvard.yaml",
    "springer-basic-author-date-no-et-al.yaml",
    "the-company-of-biologists.yaml",
    "the-geological-society-of-london.yaml",
]


def deduplicate_sort_template(content):
    """
    Deduplicate keys in any sort.template section.
    Works on both options.processing.sort and bibliography.sort.
    Removes consecutive duplicate key entries, keeping first occurrence.
    """
    lines = content.split('\n')
    output = []
    in_sort_template = False
    seen_keys = set()
    i = 0

    while i < len(lines):
        line = lines[i]

        # Detect when we enter a sort.template section
        if line.strip() == 'template:' and i > 0:
            # Check if previous non-empty line is 'sort:'
            j = i - 1
            while j >= 0 and lines[j].strip() == '':
                j -= 1
            if j >= 0 and lines[j].strip() == 'sort:':
                output.append(line)
                i += 1
                in_sort_template = True
                seen_keys.clear()
                continue

        # Detect exit from sort template
        if in_sort_template and line and not line.startswith('      '):
            # We exit when we hit a line that's not indented with 6+ spaces
            in_sort_template = False
            seen_keys.clear()

        # Process items in sort template
        if in_sort_template:
            # Lines starting with '      - key:' are our template items
            if line.startswith('      - key:'):
                key_value = line.split('- key:')[1].strip()

                # Check if this key was already seen in this template
                if key_value in seen_keys:
                    # Skip this line and the next line (ascending flag)
                    i += 1
                    if i < len(lines) and lines[i].startswith('        ascending:'):
                        i += 1
                    continue
                else:
                    seen_keys.add(key_value)

        output.append(line)
        i += 1

    return '\n'.join(output)


def main():
    repo_root = Path(__file__).parent.parent
    styles_dir = repo_root / "styles"

    changed_count = 0

    for style_file in AFFECTED_STYLES:
        style_path = styles_dir / style_file

        if not style_path.exists():
            print(f"⚠️  {style_file} not found, skipping")
            continue

        with open(style_path, 'r') as f:
            original_content = f.read()

        deduplicated_content = deduplicate_sort_template(original_content)

        if original_content != deduplicated_content:
            with open(style_path, 'w') as f:
                f.write(deduplicated_content)
            print(f"✓ {style_file}")
            changed_count += 1
        else:
            print(f"  {style_file} (no duplicates found)")

    print(f"\nDeduplication complete: {changed_count} files updated")


if __name__ == "__main__":
    main()
