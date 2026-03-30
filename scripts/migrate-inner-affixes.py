#!/usr/bin/env python3
"""
Migrate YAML styles from flat inner-prefix/inner-suffix to nested WrapConfig.

Before:
  inner-prefix: " "
  inner-suffix: .
  wrap: parentheses

After:
  wrap:
    punctuation: parentheses
    inner-prefix: " "
    inner-suffix: .
"""

import re
from pathlib import Path


def migrate_yaml_file(file_path):
    """
    Migrate a single YAML file from flat to nested wrap format.

    Returns True if the file was modified, False otherwise.
    """
    with open(file_path, 'r') as f:
        lines = f.readlines()

    modified = False
    output_lines = []
    i = 0

    while i < len(lines):
        line = lines[i]

        # Look for wrap: <scalar> pattern at start of line (after indentation)
        match = re.match(r'^(\s*)(wrap):\s*(\S+)\s*$', line)
        if match:
            indent = match.group(1)
            wrap_value = match.group(3)

            # Collect the wrap line and look ahead for inner-prefix/inner-suffix
            collected_inner = {}
            j = i + 1

            # Look ahead for inner-prefix and inner-suffix at the same indentation level
            while j < len(lines):
                next_line = lines[j]

                # Check if we're still at the same indentation level
                next_indent_match = re.match(r'^(\s*)', next_line)
                next_indent = next_indent_match.group(1) if next_indent_match else ''

                # Stop if we encounter a line at a different indentation (less indent = end of this block)
                if next_indent and len(next_indent) < len(indent):
                    break

                # Only process lines at the exact same indentation
                if next_indent == indent:
                    inner_prefix_match = re.match(
                        r'^(\s*)inner-prefix:\s*(.*)$',
                        next_line
                    )
                    inner_suffix_match = re.match(
                        r'^(\s*)inner-suffix:\s*(.*)$',
                        next_line
                    )

                    if inner_prefix_match:
                        collected_inner['inner-prefix'] = inner_prefix_match.group(2)
                        j += 1
                        continue
                    elif inner_suffix_match:
                        collected_inner['inner-suffix'] = inner_suffix_match.group(2)
                        j += 1
                        continue

                # If we encounter any other key at this indentation, stop looking
                if next_indent == indent and re.match(r'^(\s*)\S+:', next_line):
                    break

                # Skip empty or comment lines
                if re.match(r'^\s*(?:#|$)', next_line):
                    j += 1
                    continue

                # Line at greater indentation (child of wrap) - stop looking
                if next_indent and len(next_indent) > len(indent):
                    break

                j += 1

            # If we found inner-prefix/suffix, convert to nested format
            if collected_inner:
                # Emit the nested wrap block
                output_lines.append(f"{indent}wrap:\n")
                output_lines.append(f"{indent}  punctuation: {wrap_value}\n")

                if 'inner-prefix' in collected_inner:
                    output_lines.append(f"{indent}  inner-prefix: {collected_inner['inner-prefix']}\n")
                if 'inner-suffix' in collected_inner:
                    output_lines.append(f"{indent}  inner-suffix: {collected_inner['inner-suffix']}\n")

                # Skip the inner-prefix/suffix lines we've processed
                i = j
                modified = True
            else:
                # No inner affixes found, but still convert to nested format for consistency
                output_lines.append(f"{indent}wrap:\n")
                output_lines.append(f"{indent}  punctuation: {wrap_value}\n")
                i += 1
                modified = True
        else:
            output_lines.append(line)
            i += 1

    if modified:
        with open(file_path, 'w') as f:
            f.writelines(output_lines)

    return modified


def main():
    """Migrate all YAML files in styles/ directory."""
    styles_dir = Path(__file__).resolve().parent.parent / 'styles'

    if not styles_dir.exists():
        print(f"Error: styles directory not found at {styles_dir}")
        return

    modified_files = []

    for yaml_file in sorted(styles_dir.rglob('*.yaml')):
        if migrate_yaml_file(yaml_file):
            modified_files.append(yaml_file)
            print(f"Migrated: {yaml_file.relative_to(styles_dir.parent)}")

    if modified_files:
        print(f"\nTotal files modified: {len(modified_files)}")
    else:
        print("No files needed migration.")


if __name__ == '__main__':
    main()
