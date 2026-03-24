#!/usr/bin/env python3
"""
Convert per-component `overrides:` to spec-level `type-variants:` in Citum YAML styles.

This script transforms the old v1 overrides syntax to the new v2 type-variants syntax.
For each template section (citation/bibliography and their sub-specs), it:
  1. Collects all unique type selectors from component overrides
  2. For each type, builds a complete effective template
  3. Writes type-variants at the spec level
  4. Removes overrides (and redundant suppress) from all default-template components
"""

import sys
import io
import copy
from pathlib import Path
from ruamel.yaml import YAML
from ruamel.yaml.comments import CommentedMap, CommentedSeq


def get_type_names(key) -> list[str]:
    """Extract type name(s) from a YAML overrides key (string or tuple/list)."""
    if isinstance(key, str):
        return [key]
    elif isinstance(key, (list, tuple)):
        return list(key)
    else:
        return [str(key)]


def collect_all_types(template: list) -> list[str]:
    """Collect all unique type names from overrides in a template."""
    types = set()
    for comp in template:
        if not isinstance(comp, dict):
            continue
        overrides = comp.get('overrides')
        if overrides:
            for key in overrides.keys():
                types.update(get_type_names(key))
        # Always recurse into nested groups (even if outer comp has no overrides)
        for nested_key in ('group', 'items'):
            if nested_key in comp:
                types.update(collect_all_types(comp[nested_key]))
    return sorted(types)


def get_override_for_type(overrides: dict, type_name: str) -> dict | None:
    """Find the override value for a specific type name, expanding group selectors."""
    # First exact match
    for key, val in overrides.items():
        names = get_type_names(key)
        if type_name in names:
            return val
    
    # Then fallback to 'all' or 'default'
    for key, val in overrides.items():
        names = get_type_names(key)
        if 'all' in names or 'default' in names:
            return val
            
    return None


def apply_override_to_comp(comp: dict, type_name: str) -> dict | None:
    """
    Apply type-specific override to a component.
    Returns None if component is suppressed for this type.
    Returns modified component dict without overrides key.
    """
    overrides = comp.get('overrides', {})
    override_val = get_override_for_type(overrides, type_name)

    # Base component without overrides and without suppress
    base = {k: v for k, v in comp.items() if k not in ('overrides',)}
    base_suppressed = base.pop('suppress', False)

    if override_val is None:
        # No override for this type — use base, respecting base suppress
        if base_suppressed:
            return None
        return base

    # Override found
    if isinstance(override_val, dict):
        ov_suppress = override_val.get('suppress', None)
        if ov_suppress is True:
            return None  # Suppressed for this type
        if ov_suppress is False:
            # Explicitly unsuppressed (base might be suppressed)
            pass

        # Merge override into base
        merged = dict(base)
        for k, v in override_val.items():
            if k == 'suppress':
                continue  # handled above
            merged[k] = v
        return merged
    else:
        # Primitive override (shouldn't happen in practice)
        if base_suppressed:
            return None
        return base


def build_type_template(template: list, type_name: str) -> list | None:
    """
    Build the effective template for a specific type.
    Returns None if result is empty.
    """
    result = []
    for comp in template:
        if not isinstance(comp, dict):
            result.append(comp)
            continue

        effective = apply_override_to_comp(comp, type_name)
        if effective is None:
            continue

        nested_key = 'group' if 'group' in effective else ('items' if 'items' in effective else None)
        if nested_key and has_overrides_anywhere(comp):
            nested = effective[nested_key]
            nested_result = build_type_template(nested, type_name)
            if nested_result is None or len(nested_result) == 0:
                # Skip empty groups
                continue
            effective[nested_key] = nested_result
        
        result.append(effective)

    return result if result else None


def build_default_template(template: list) -> list:
    """
    Build the default template by applying the 'default' override logic.
    """
    res = build_type_template(template, "default")
    return res if res is not None else []


def has_overrides_anywhere(comp) -> bool:
    """Check if a component or any nested component has overrides."""
    if not isinstance(comp, dict):
        return False
    if 'overrides' in comp:
        return True
    for key in ('group', 'items'):
        if key in comp:
            for nested in comp[key]:
                if has_overrides_anywhere(nested):
                    return True
    return False


def templates_equal(a: list, b: list) -> bool:
    """Compare two templates for equality, ignoring ordering of dict keys."""
    if len(a) != len(b):
        return False
    for ca, cb in zip(a, b):
        if ca != cb:
            return False
    return True


def clean_template(template: list) -> list:
    """
    Clean a template by removing overrides and redundant suppress fields from all
    components (including nested groups). Used to sanitize existing type-variants entries.
    """
    return build_default_template(template)


def convert_template_section(spec: dict) -> dict:
    """
    Convert a spec dict (with template: [...]) to use type-variants instead of per-component overrides.
    Also cleans overrides from any existing type-variants entries.
    """
    new_spec = CommentedMap()

    # First pass: collect all keys, clean existing type-variants entries
    for k, v in spec.items():
        if k == 'type-variants' and isinstance(v, dict):
            # Clean overrides from each existing type-variant template
            cleaned_tv = {}
            for type_name, tmpl in v.items():
                if isinstance(tmpl, list):
                    cleaned_tv[type_name] = clean_template(tmpl)
                else:
                    cleaned_tv[type_name] = tmpl
            new_spec[k] = cleaned_tv
        else:
            new_spec[k] = v

    template = new_spec.get('template')
    if not template:
        return new_spec

    # Collect all types mentioned in overrides
    all_types = collect_all_types(template)
    if not all_types:
        # No overrides in template, but still clean default suppress fields
        new_spec['template'] = build_default_template(template)
        return new_spec

    # Build clean default template
    default_tmpl = build_default_template(template)

    # Merge with existing type-variants (if any)
    existing_tv = dict(new_spec.get('type-variants') or {})

    # Build type-specific templates from template overrides
    new_type_variants = {}
    for type_name in all_types:
        type_tmpl = build_type_template(template, type_name)
        if type_tmpl is None or len(type_tmpl) == 0:
            # Empty template = suppress entirely for this type
            new_type_variants[type_name] = []
        elif templates_equal(type_tmpl, default_tmpl):
            # Same as default — no need for type-variants entry
            continue
        else:
            new_type_variants[type_name] = type_tmpl

    # Merge: existing type-variants take precedence (they were hand-authored)
    # New type-variants only added if not already in existing
    merged_tv = {}
    merged_tv.update(new_type_variants)
    merged_tv.update(existing_tv)  # existing wins for same keys

    if not merged_tv:
        new_spec['template'] = default_tmpl
        return new_spec

    # Build the new spec with type-variants inserted before template
    result = CommentedMap()
    inserted_tv = False
    for k, v in new_spec.items():
        if k == 'type-variants':
            result['type-variants'] = merged_tv
            inserted_tv = True
        elif k == 'template':
            if not inserted_tv and merged_tv:
                result['type-variants'] = merged_tv
                inserted_tv = True
            result['template'] = default_tmpl
        else:
            result[k] = v
    if not inserted_tv and merged_tv:
        result['type-variants'] = merged_tv
    if 'template' not in result:
        result['template'] = default_tmpl

    return result


def process_citation_spec(cit: dict) -> dict:
    """Process a citation spec, handling nested integral/non-integral sub-specs."""
    if not isinstance(cit, dict):
        return cit

    new_cit = CommentedMap(cit)

    # Top-level template
    if 'template' in cit:
        new_cit = convert_template_section(dict(cit))

    # Sub-specs: integral, non-integral, subsequent, ibid
    for sub_key in ('integral', 'non-integral', 'subsequent', 'ibid'):
        if sub_key in new_cit and isinstance(new_cit[sub_key], dict):
            new_cit[sub_key] = convert_template_section(dict(new_cit[sub_key]))

    return new_cit


def process_bibliography_spec(bib: dict) -> dict:
    """Process a bibliography spec."""
    if not isinstance(bib, dict):
        return bib
    return convert_template_section(dict(bib))


def convert_style(data: dict) -> dict:
    """Convert a full style dict from overrides to type-variants."""
    new_data = CommentedMap(data)

    if 'citation' in new_data:
        new_data['citation'] = process_citation_spec(dict(new_data['citation']))

    if 'bibliography' in new_data:
        new_data['bibliography'] = process_bibliography_spec(dict(new_data['bibliography']))

    return new_data


def process_file(path: Path, dry_run: bool = False) -> bool:
    """Process a single YAML file. Returns True if modified."""
    yaml = YAML()
    yaml.preserve_quotes = True
    yaml.width = 100
    yaml.best_map_flow_style = False

    with open(path) as f:
        data = yaml.load(f)

    if data is None:
        return False

    # Quick check: does it have overrides anywhere?
    content = path.read_text()
    if 'overrides:' not in content:
        return False

    new_data = convert_style(data)

    if dry_run:
        buf = io.StringIO()
        yaml.dump(new_data, buf)
        print(f"=== {path} ===")
        print(buf.getvalue())
        return True

    with open(path, 'w') as f:
        yaml.dump(new_data, f)

    return True


def main():
    import argparse
    parser = argparse.ArgumentParser(description='Convert overrides to type-variants')
    parser.add_argument('files', nargs='*', help='YAML style files to convert')
    parser.add_argument('--dry-run', action='store_true', help='Print output without writing')
    parser.add_argument('--styles-dir', default='styles', help='Directory containing styles')
    args = parser.parse_args()

    if args.files:
        paths = [Path(f) for f in args.files]
    else:
        paths = sorted(Path(args.styles_dir).rglob('*.yaml'))

    modified = 0
    for path in paths:
        try:
            changed = process_file(path, dry_run=args.dry_run)
            if changed:
                modified += 1
                if not args.dry_run:
                    print(f'Converted: {path}')
        except Exception as e:
            print(f'ERROR {path}: {e}', file=sys.stderr)

    print(f'\n{"Would modify" if args.dry_run else "Modified"} {modified} files')


if __name__ == '__main__':
    main()
