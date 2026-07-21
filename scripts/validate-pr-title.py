#!/usr/bin/env python3
"""Validate a squash-merge pull request title against the alint commit policy."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
ALINT_CONFIG = REPO_ROOT / ".alint.yml"


def commit_subject_policy(config_path: Path = ALINT_CONFIG) -> tuple[re.Pattern[str], int]:
    """Return the conventional-commit pattern and subject limit from alint."""
    config = config_path.read_text(encoding="utf-8")
    rule = re.search(
        r"(?ms)^  - id: conventional-commits\n(?P<body>.*?)(?=^  - id:|\Z)",
        config,
    )
    if rule is None:
        raise ValueError(".alint.yml does not define the conventional-commits rule")

    pattern = re.search(r"(?m)^    pattern: '(.+)'$", rule["body"])
    limit = re.search(r"(?m)^    subject_max_length: (\d+)$", rule["body"])
    if pattern is None or limit is None:
        raise ValueError("conventional-commits is missing pattern or subject_max_length")

    return re.compile(pattern[1]), int(limit[1])


def validate_title(title: str) -> str | None:
    """Return a validation error when a pull request title cannot be a commit subject."""
    pattern, maximum_length = commit_subject_policy()
    if "\n" in title or "\r" in title:
        return "pull request title must be a single line"
    if len(title) > maximum_length:
        return f"pull request title is {len(title)} chars; max allowed is {maximum_length}"
    if pattern.fullmatch(title) is None:
        return "pull request title does not match the conventional-commit policy"
    return None


def main() -> int:
    """Validate the title supplied by GitHub Actions."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("title", help="pull request title to validate")
    args = parser.parse_args()

    error = validate_title(args.title)
    if error is None:
        return 0

    print(f"[ERROR] {error}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
