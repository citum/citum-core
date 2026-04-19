#!/usr/bin/env python3
"""Validate committed workspace version metadata against Version-Bump footers."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Sequence


REPO_ROOT = Path(__file__).resolve().parent.parent
CARGO_TOML = REPO_ROOT / "Cargo.toml"

FIELD_SEPARATOR = "\x1f"
RECORD_SEPARATOR = "\x1e"
WORKSPACE_VERSION_RE = re.compile(
    r"\[workspace\.package\][^\[]*?version\s*=\s*\"([^\"]+)\"",
    re.DOTALL,
)
VERSION_BUMP_RE = re.compile(r"^Version-Bump:\s*(patch|minor|major)\s*$", re.MULTILINE)

PUBLISHABLE_PREFIXES = (
    "crates/csl-legacy/",
    "crates/citum-schema-data/",
    "crates/citum-schema-style/",
    "crates/citum-schema/",
    "crates/citum-migrate/",
    "crates/citum-engine/",
    "crates/citum-cli/",
    "crates/citum-bindings/",
)


class ValidationError(RuntimeError):
    """Raised when version release validation cannot proceed."""


@dataclass(frozen=True)
class VersionBumpMarker:
    """One Version-Bump footer discovered in the unreleased commit range."""

    commit: str
    subject: str
    bump: str


def is_publishable_path(path: str) -> bool:
    """Return true when a changed path is in a publishable crate."""
    return path.endswith(".rs") and any(path.startswith(p) for p in PUBLISHABLE_PREFIXES)


def is_version_metadata_path(path: str) -> bool:
    """Return true when a changed path is part of version bookkeeping."""
    return path in {"Cargo.toml", "Cargo.lock"}


def run(
    args: Sequence[str],
    *,
    capture_output: bool = True,
    check: bool = True,
    cwd: Path = REPO_ROOT,
) -> subprocess.CompletedProcess[str]:
    """Run a command in the repository root."""
    return subprocess.run(
        list(args),
        cwd=cwd,
        text=True,
        capture_output=capture_output,
        check=check,
    )


def list_changed_files(commit_range: str) -> list[str]:
    """Return the changed files in the validated commit range."""
    result = run(["git", "diff", "--name-only", commit_range])
    return [line.strip() for line in result.stdout.splitlines() if line.strip()]


def read_workspace_version_from_text(content: str) -> str:
    """Extract the workspace version from Cargo.toml content."""
    match = WORKSPACE_VERSION_RE.search(content)
    if match is None:
        raise ValidationError(
            "Could not determine [workspace.package].version from Cargo.toml"
        )
    return match.group(1)


def read_current_workspace_version() -> str:
    """Read the current workspace version from disk."""
    return read_workspace_version_from_text(CARGO_TOML.read_text(encoding="utf-8"))


def read_workspace_version_at_ref(ref: str) -> str:
    """Read the workspace version at a git ref."""
    result = run(["git", "show", f"{ref}:Cargo.toml"])
    return read_workspace_version_from_text(result.stdout)


def bump_version(current: str, bump_type: str) -> str:
    """Return the semantic version produced by the bump type."""
    major_s, minor_s, patch_s = current.split(".")
    major, minor, patch = int(major_s), int(minor_s), int(patch_s)
    if bump_type == "patch":
        patch += 1
    elif bump_type == "minor":
        minor += 1
        patch = 0
    elif bump_type == "major":
        if major == 0:
            # Pre-1.0: treat breaking changes as minor bumps.
            minor += 1
        else:
            major += 1
            minor = 0
        patch = 0
    else:
        raise ValidationError(f"Unsupported bump type: {bump_type}")
    return f"{major}.{minor}.{patch}"


def collect_version_bump_markers(commit_range: str) -> list[VersionBumpMarker]:
    """Collect Version-Bump footers from the unreleased commit range."""
    result = run(
        [
            "git",
            "log",
            f"--format=%H{FIELD_SEPARATOR}%s{FIELD_SEPARATOR}%B{RECORD_SEPARATOR}",
            commit_range,
        ]
    )
    markers: list[VersionBumpMarker] = []
    records = result.stdout.strip(RECORD_SEPARATOR)
    if not records:
        return markers

    for record in records.split(RECORD_SEPARATOR):
        if not record.strip():
            continue
        commit, subject, body = record.split(FIELD_SEPARATOR, maxsplit=2)
        bump_matches = VERSION_BUMP_RE.findall(body)
        if len(bump_matches) > 1:
            raise ValidationError(
                f"Commit {commit[:8]} ({subject}) declares multiple Version-Bump footers"
            )
        if bump_matches:
            markers.append(
                VersionBumpMarker(commit=commit[:8], subject=subject, bump=bump_matches[0])
            )
    return markers


def format_marker_list(markers: Sequence[VersionBumpMarker]) -> str:
    """Render a short marker summary for error messages."""
    return ", ".join(f"{m.commit} ({m.bump})" for m in markers)


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    """Parse command-line arguments."""
    parser = argparse.ArgumentParser(
        prog="validate-version-release.py",
        description="Validate committed workspace version against Version-Bump footers.",
    )
    baseline_group = parser.add_mutually_exclusive_group(required=True)
    baseline_group.add_argument(
        "--previous-tag",
        help="Latest released root tag (for example v0.20.0).",
    )
    baseline_group.add_argument(
        "--baseline-ref",
        help="Git ref used as the comparison baseline (e.g. origin/main or a PR base SHA).",
    )
    parser.add_argument(
        "--commit-range",
        help="Explicit git commit range to scan. Defaults to <baseline-ref>..HEAD.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Accepted for backward compatibility. Validation is always non-mutating.",
    )
    parser.add_argument(
        "--allow-orphan-footer",
        action="store_true",
        help=(
            "Allow exactly one Version-Bump footer even when the validated range does "
            "not change publishable crate source or workspace version."
        ),
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str]) -> int:
    """Validate committed workspace version metadata."""
    try:
        args = parse_args(argv)
        baseline_ref = args.previous_tag or args.baseline_ref
        commit_range = args.commit_range or f"{baseline_ref}..HEAD"

        changed_files = list_changed_files(commit_range)
        publishable_changed = any(is_publishable_path(f) for f in changed_files)
        version_metadata_changed = any(is_version_metadata_path(f) for f in changed_files)
        relevant_change = publishable_changed or version_metadata_changed

        try:
            previous_version = read_workspace_version_at_ref(baseline_ref)
        except subprocess.CalledProcessError:
            print(f"Warning: could not read workspace version at {baseline_ref}; skipping.")
            return 0

        current_version = read_current_workspace_version()
        version_changed = current_version != previous_version
        markers = collect_version_bump_markers(commit_range)

        if not relevant_change and not version_changed:
            if markers:
                if args.allow_orphan_footer and len(markers) == 1:
                    marker = markers[0]
                    print(
                        f"No version validation change needed; allowing rescue footer "
                        f"{marker.bump} from {marker.commit}."
                    )
                    return 0
                raise ValidationError(
                    "Found Version-Bump footer(s) but no publishable crate changed "
                    f"and workspace version is unchanged: {format_marker_list(markers)}"
                )
            print(
                f"No workspace version changes since {baseline_ref}; "
                "publishable crate sources and version are unchanged."
            )
            return 0

        if version_changed and not markers:
            raise ValidationError(
                f"Workspace version changed ({previous_version} → {current_version}) "
                f"but no Version-Bump footer found across {commit_range}."
            )

        if markers and not version_changed:
            raise ValidationError(
                "Found Version-Bump footer(s) but workspace version did not change "
                f"from {previous_version}: {format_marker_list(markers)}"
            )

        if not markers:
            raise ValidationError(
                "Publishable crate files or version metadata changed "
                f"across {commit_range} but no Version-Bump footer was found."
            )

        bump_order = {"patch": 0, "minor": 1, "major": 2}
        marker = max(markers, key=lambda m: bump_order.get(m.bump, -1))
        if len(markers) > 1:
            print(
                f"Multiple Version-Bump footers; using highest severity "
                f"({marker.bump} from {marker.commit}): {format_marker_list(markers)}"
            )

        expected_version = bump_version(previous_version, marker.bump)
        if current_version != expected_version:
            raise ValidationError(
                "Workspace version does not match the declared Version-Bump footer. "
                f"Expected {expected_version} (from {marker.bump} bump of {previous_version}), "
                f"found {current_version}."
            )

        print(
            f"Version metadata validated: {previous_version} → {current_version}, "
            f"Version-Bump: {marker.bump} from {marker.commit}."
        )
        return 0

    except ValidationError as exc:
        print(f"error: {exc}", flush=True)
        return 1
    except subprocess.CalledProcessError as exc:
        stderr = exc.stderr.strip() if exc.stderr else ""
        stdout = exc.stdout.strip() if exc.stdout else ""
        print(f"error: {stderr or stdout or exc}", flush=True)
        return exc.returncode or 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
