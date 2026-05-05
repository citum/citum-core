#!/usr/bin/env python3
"""Infer release impact from conventional commits and changed paths."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Sequence


REPO_ROOT = Path(__file__).resolve().parent.parent
LEVEL_ORDER = {"none": 0, "patch": 1, "minor": 2, "major": 3}
CODE_PATHS = ("crates/", "Cargo.toml", "Cargo.lock")
SCHEMA_PATHS = (
    "crates/citum-schema-data/",
    "crates/citum-schema-style/",
    "crates/citum-schema/",
    "crates/citum-cli/",
    "docs/schemas/",
)


@dataclass(frozen=True)
class ReleaseImpact:
    """Resolved release impact for one commit range."""

    level: str
    should_release: bool
    code_changed: bool
    schema_changed: bool


def run_git(args: Sequence[str]) -> str:
    """Run a git command in the repository root and return stdout."""

    result = subprocess.run(
        ["git", *args],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=True,
    )
    return result.stdout


def changed_files(commit_range: str) -> list[str]:
    """Return files changed by the supplied git revision range."""

    if ".." in commit_range:
        output = run_git(["diff", "--name-only", commit_range])
    else:
        output = run_git(["show", "--format=", "--name-only", commit_range])
    return [line.strip() for line in output.splitlines() if line.strip()]


def commit_subjects(commit_range: str) -> list[str]:
    """Return commit subjects in the supplied git revision range."""

    output = run_git(["log", "--format=%s", commit_range])
    return [line.strip() for line in output.splitlines() if line.strip()]


def commit_bodies(commit_range: str) -> str:
    """Return commit bodies in the supplied git revision range."""

    return run_git(["log", "--format=%b", commit_range])


def is_code_path(path: str) -> bool:
    """Return whether a path should trigger a workspace crate release."""

    return path in CODE_PATHS or path.startswith(CODE_PATHS)


def is_schema_path(path: str) -> bool:
    """Return whether a path should trigger a schema version bump."""

    return path.startswith(SCHEMA_PATHS)


def cap_major_for_pre_one(level: str, current_version: str) -> str:
    """Apply the repository's pre-1.0 major-as-minor policy."""

    if level == "major" and current_version.split(".", maxsplit=1)[0] == "0":
        return "minor"
    return level


def conventional_level(subjects: Sequence[str], bodies: str, current_version: str) -> str:
    """Infer the highest release bump from conventional commit messages."""

    level = "none"
    for subject in subjects:
        if re.match(r"^(feat|fix|perf)(\([^)]+\))?!:", subject):
            level = "major"
            break
        if re.match(r"^feat(\([^)]+\))?:", subject):
            level = max(level, "minor", key=LEVEL_ORDER.__getitem__)
        elif re.match(r"^(fix|perf)(\([^)]+\))?:", subject):
            level = max(level, "patch", key=LEVEL_ORDER.__getitem__)

    if re.search(r"^BREAKING CHANGE:", bodies, flags=re.MULTILINE):
        level = "major"

    return cap_major_for_pre_one(level, current_version)


def infer_release_impact(
    paths: Sequence[str],
    subjects: Sequence[str],
    bodies: str,
    current_version: str,
) -> ReleaseImpact:
    """Infer release impact from paths plus conventional commit metadata."""

    code_changed = any(is_code_path(path) for path in paths)
    schema_changed = any(is_schema_path(path) for path in paths)
    level = conventional_level(subjects, bodies, current_version)
    should_release = (code_changed or schema_changed) and level != "none"
    if not should_release:
        level = "none"
    return ReleaseImpact(
        level=level,
        should_release=should_release,
        code_changed=code_changed,
        schema_changed=schema_changed,
    )


def write_github_output(path: Path, impact: ReleaseImpact) -> None:
    """Append release impact fields to a GitHub Actions output file."""

    with path.open("a", encoding="utf-8") as output:
        output.write(f"level={impact.level}\n")
        output.write(f"should-release={str(impact.should_release).lower()}\n")
        output.write(f"code-changed={str(impact.code_changed).lower()}\n")
        output.write(f"schema-changed={str(impact.schema_changed).lower()}\n")


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    """Parse command-line arguments."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--range", required=True, help="Git revision range to inspect.")
    parser.add_argument(
        "--current-version",
        required=True,
        help="Current workspace version used for pre-1.0 major capping.",
    )
    parser.add_argument(
        "--github-output",
        type=Path,
        help="Optional GitHub Actions output file path.",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str]) -> int:
    """Run the release inference CLI."""

    args = parse_args(argv)
    impact = infer_release_impact(
        changed_files(args.range),
        commit_subjects(args.range),
        commit_bodies(args.range),
        args.current_version,
    )
    if args.github_output is not None:
        write_github_output(args.github_output, impact)
    print(json.dumps(impact.__dict__, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
