#!/usr/bin/env python3
"""Synchronize shipped Citum style versions with STYLE_SCHEMA_VERSION."""

from __future__ import annotations

import argparse
import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
VERSION_SOURCE = REPO_ROOT / "crates/citum-schema-style/src/version.rs"
STYLE_GLOBS = (
    "styles/*.yaml",
    "crates/citum-schema-style/embedded/styles/*.yaml",
)
VERSION_PATTERN = re.compile(r'^pub const STYLE_SCHEMA_VERSION: &str = "([^"]+)";', re.MULTILINE)
STYLE_VERSION_PATTERN = re.compile(r"^version:\s*.*$", re.MULTILINE)


def schema_version() -> str:
    """Read the canonical style schema version from Rust source."""
    match = VERSION_PATTERN.search(VERSION_SOURCE.read_text(encoding="utf-8"))
    if match is None:
        raise SystemExit(f"STYLE_SCHEMA_VERSION not found in {VERSION_SOURCE}")
    return match.group(1)


def shipped_styles() -> list[Path]:
    """Return every canonical shipped style without following mirror symlinks."""
    return sorted({path for pattern in STYLE_GLOBS for path in REPO_ROOT.glob(pattern)})


def synchronized_text(path: Path, version: str) -> str:
    """Return style text with exactly the canonical top-level version."""
    text = path.read_text(encoding="utf-8")
    replacement = f'version: "{version}"'
    if STYLE_VERSION_PATTERN.search(text):
        return STYLE_VERSION_PATTERN.sub(replacement, text, count=1)

    lines = text.splitlines(keepends=True)
    insert_at = 1 if lines and lines[0].startswith("# yaml-language-server:") else 0
    lines.insert(insert_at, f"{replacement}\n")
    return "".join(lines)


def main() -> int:
    """Update shipped styles, or report drift in check mode."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="fail instead of rewriting drift")
    args = parser.parse_args()
    version = schema_version()
    stale: list[Path] = []

    for path in shipped_styles():
        current = path.read_text(encoding="utf-8")
        expected = synchronized_text(path, version)
        if current == expected:
            continue
        stale.append(path.relative_to(REPO_ROOT))
        if not args.check:
            path.write_text(expected, encoding="utf-8")

    if args.check and stale:
        print(f"Shipped styles are not synchronized to schema {version}:")
        for path in stale:
            print(f"  {path}")
        return 1

    action = "checked" if args.check else "synchronized"
    print(f"{action} {len(shipped_styles())} shipped styles at schema {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
