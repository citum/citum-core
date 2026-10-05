#!/usr/bin/env python3
"""Pin the embedded registry to immutable core and catalog revisions."""

from __future__ import annotations

import argparse
import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
REGISTRY = REPO_ROOT / "crates/citum-schema-style/embedded/registry/default.yaml"
CARGO_TOML = REPO_ROOT / "Cargo.toml"
CATALOG_REF = "3592af76f7e9ff69e3714f6582b812306ff438f3"
CORE_URL = re.compile(r"(https://raw\.githubusercontent\.com/citum/citum-core/)([^/]+)(/styles/)")
CATALOG_URL = re.compile(r"(https://raw\.githubusercontent\.com/citum/citum-styles/)([^/]+)(/styles/)")
WORKSPACE_VERSION = re.compile(
    r'^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"', re.MULTILINE | re.DOTALL
)


def current_core_ref() -> str:
    """Return the release tag matching the workspace version."""
    match = WORKSPACE_VERSION.search(CARGO_TOML.read_text(encoding="utf-8"))
    if match is None:
        raise SystemExit("workspace package version not found in Cargo.toml")
    return f"v{match.group(1)}"


def pinned_registry(text: str, core_ref: str) -> str:
    """Rewrite known GitHub sources to immutable revisions."""
    text = CORE_URL.sub(rf"\g<1>{core_ref}\g<3>", text)
    return CATALOG_URL.sub(rf"\g<1>{CATALOG_REF}\g<3>", text)


def main() -> int:
    """Rewrite registry pins, or fail when the checked-in registry drifts."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="fail instead of rewriting drift")
    args = parser.parse_args()
    core_ref = current_core_ref()
    current = REGISTRY.read_text(encoding="utf-8")
    expected = pinned_registry(current, core_ref)

    if current != expected:
        if args.check:
            print(
                f"registry sources must use core {core_ref} and citum-styles {CATALOG_REF}"
            )
            return 1
        REGISTRY.write_text(expected, encoding="utf-8")

    mutable = re.findall(
        r"https://raw\.githubusercontent\.com/[^\s]+/(?:main|master)/[^\s]+", expected
    )
    if mutable:
        print("mutable registry sources remain:")
        for url in mutable:
            print(f"  {url}")
        return 1

    action = "checked" if args.check else "pinned"
    print(f"{action} registry sources at core {core_ref} and catalog {CATALOG_REF}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
