#!/usr/bin/env python3
"""Validate committed schema artifacts and version metadata."""

from __future__ import annotations

import argparse
import re
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Sequence


REPO_ROOT = Path(__file__).resolve().parent.parent
SCHEMA_DIR = REPO_ROOT / "docs/schemas"
STYLE_SCHEMA_LIB = REPO_ROOT / "crates/citum-schema-style/src/lib.rs"

FIELD_SEPARATOR = "\x1f"
RECORD_SEPARATOR = "\x1e"
SCHEMA_VERSION_RE = re.compile(r'pub const STYLE_SCHEMA_VERSION: &str = "([^"]+)";')
SCHEMA_BUMP_RE = re.compile(r"^Schema-Bump:\s*(patch|minor|major)\s*$", re.MULTILINE)
SCHEMA_CORRECT_RE = re.compile(r"^Schema-Correct:\s*(\S+)\s*$", re.MULTILINE)


class ReleasePrepError(RuntimeError):
    """Raised when schema release validation cannot proceed safely."""


@dataclass(frozen=True)
class SchemaBumpMarker:
    """One schema bump footer discovered in the unreleased commit range."""

    commit: str
    subject: str
    bump: str


@dataclass(frozen=True)
class SchemaCorrectMarker:
    """A Schema-Correct footer that overrides bump logic with an explicit version."""

    commit: str
    subject: str
    version: str


@dataclass(frozen=True)
class ValidationTarget:
    """Resolved git references used to validate schema release state."""

    baseline_ref: str
    baseline_label: str
    commit_range: str


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


def read_schema_version_from_text(content: str) -> str:
    """Extract the schema version constant from source text."""

    match = SCHEMA_VERSION_RE.search(content)
    if match is None:
        raise ReleasePrepError(
            "Could not determine STYLE_SCHEMA_VERSION from crates/citum-schema-style/src/lib.rs"
        )
    return match.group(1)


def read_current_schema_version() -> str:
    """Read the current schema version constant from disk."""

    return read_schema_version_from_text(STYLE_SCHEMA_LIB.read_text(encoding="utf-8"))


def read_schema_version_at_ref(ref: str) -> str:
    """Read the schema version constant at a git ref."""

    rel_path = STYLE_SCHEMA_LIB.relative_to(REPO_ROOT).as_posix()
    result = run(["git", "show", f"{ref}:{rel_path}"])
    return read_schema_version_from_text(result.stdout)


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
            # Pre-1.0: treat breaking changes as minor bumps (mirrors release-plz behaviour).
            # A 1.0 promotion requires an explicit manual release.
            minor += 1
        else:
            major += 1
            minor = 0
        patch = 0
    else:
        raise ReleasePrepError(f"Unsupported schema bump type: {bump_type}")
    return f"{major}.{minor}.{patch}"


def export_schemas(out_dir: Path) -> None:
    """Generate JSON schemas into the target directory."""

    out_dir.mkdir(parents=True, exist_ok=True)
    run(
        [
            "cargo",
            "run",
            "--bin",
            "citum",
            "--features",
            "schema",
            "--",
            "schema",
            "--out-dir",
            str(out_dir),
        ],
        capture_output=False,
    )


def read_schema_dir_contents(path: Path) -> dict[str, str]:
    """Read all committed schema files from a directory."""

    if not path.exists():
        return {}
    return {
        schema_file.name: schema_file.read_text(encoding="utf-8")
        for schema_file in sorted(path.glob("*.json"))
    }


def read_schema_dir_contents_at_ref(ref: str) -> dict[str, str]:
    """Read committed schema files from a git ref."""

    result = run(["git", "ls-tree", "--name-only", "-r", ref, "docs/schemas"])
    files = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    contents: dict[str, str] = {}
    for file_path in files:
        file_content = run(["git", "show", f"{ref}:{file_path}"]).stdout
        contents[Path(file_path).name] = file_content
    return contents


def collect_schema_bump_markers(
    commit_range: str,
) -> tuple[list[SchemaBumpMarker], list[SchemaCorrectMarker]]:
    """Collect schema bump and correction footers from the unreleased commit range."""

    result = run(
        [
            "git",
            "log",
            f"--format=%H{FIELD_SEPARATOR}%s{FIELD_SEPARATOR}%B{RECORD_SEPARATOR}",
            commit_range,
        ]
    )
    bump_markers: list[SchemaBumpMarker] = []
    correct_markers: list[SchemaCorrectMarker] = []
    records = result.stdout.strip(RECORD_SEPARATOR)
    if not records:
        return bump_markers, correct_markers

    for record in records.split(RECORD_SEPARATOR):
        if not record.strip():
            continue
        commit, subject, body = record.split(FIELD_SEPARATOR, maxsplit=2)
        bump_matches = SCHEMA_BUMP_RE.findall(body)
        correct_matches = SCHEMA_CORRECT_RE.findall(body)
        if len(bump_matches) > 1:
            raise ReleasePrepError(
                f"Commit {commit[:8]} ({subject}) declares multiple Schema-Bump footers"
            )
        if len(correct_matches) > 1:
            raise ReleasePrepError(
                f"Commit {commit[:8]} ({subject}) declares multiple Schema-Correct footers"
            )
        if bump_matches and correct_matches:
            raise ReleasePrepError(
                f"Commit {commit[:8]} ({subject}) declares both Schema-Bump and Schema-Correct footers"
            )
        if bump_matches:
            bump_markers.append(
                SchemaBumpMarker(commit=commit[:8], subject=subject, bump=bump_matches[0])
            )
        if correct_matches:
            correct_markers.append(
                SchemaCorrectMarker(commit=commit[:8], subject=subject, version=correct_matches[0])
            )
    return bump_markers, correct_markers


def format_marker_list(markers: Sequence[SchemaBumpMarker]) -> str:
    """Render a short marker summary for error messages."""

    return ", ".join(f"{marker.commit} ({marker.bump})" for marker in markers)


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    """Parse command-line arguments."""

    parser = argparse.ArgumentParser(
        prog="validate-schema-release.py",
        description="Validate committed schema artifacts against the current source tree.",
    )
    baseline_group = parser.add_mutually_exclusive_group(required=True)
    baseline_group.add_argument(
        "--previous-tag",
        help="Latest released root tag (for example v0.15.0).",
    )
    baseline_group.add_argument(
        "--baseline-ref",
        help=(
            "Git ref used as the comparison baseline for validation-only workflows "
            "(for example origin/main or a pull request base SHA)."
        ),
    )
    parser.add_argument(
        "--commit-range",
        help=(
            "Explicit git commit range to scan for Schema-Bump footers. "
            "Defaults to <baseline-ref>..HEAD."
        ),
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
            "Allow exactly one Schema-Bump footer even when the validated range does not "
            "change generated schemas or STYLE_SCHEMA_VERSION. Intended for rescue PRs "
            "whose merge commit will unblock an already schema-changing release range."
        ),
    )
    return parser.parse_args(argv)


def resolve_validation_target(args: argparse.Namespace) -> ValidationTarget:
    """Resolve the baseline ref and commit range for this validation run."""

    baseline_ref = args.previous_tag or args.baseline_ref
    if baseline_ref is None:
        raise ReleasePrepError("A baseline ref is required for release preparation")

    baseline_label = (
        args.previous_tag if args.previous_tag is not None else f"baseline {baseline_ref}"
    )
    commit_range = args.commit_range or f"{baseline_ref}..HEAD"
    return ValidationTarget(
        baseline_ref=baseline_ref,
        baseline_label=baseline_label,
        commit_range=commit_range,
    )


def main(argv: Sequence[str]) -> int:
    """Validate committed schema artifacts and schema-version metadata."""

    try:
        args = parse_args(argv)
        target = resolve_validation_target(args)

        previous_schema_version = read_schema_version_at_ref(target.baseline_ref)
        current_schema_version = read_current_schema_version()
        previous_schema_output = read_schema_dir_contents_at_ref(target.baseline_ref)
        current_schema_output = read_schema_dir_contents(SCHEMA_DIR)

        with tempfile.TemporaryDirectory(prefix="citum-schemas-") as tmp_dir:
            generated_schema_dir = Path(tmp_dir) / "schemas"
            export_schemas(generated_schema_dir)
            generated_schema_output = read_schema_dir_contents(generated_schema_dir)

            schema_output_drift = generated_schema_output != current_schema_output
            schema_files_changed_since_tag = generated_schema_output != previous_schema_output

        if schema_output_drift:
            raise ReleasePrepError(
                "Committed docs/schemas/* do not match freshly generated schemas. "
                "Run `cargo run --bin citum --features schema -- schema --out-dir docs/schemas` "
                "and commit the updated files."
            )

        schema_version_changed = current_schema_version != previous_schema_version
        schema_changed = (
            schema_output_drift or schema_files_changed_since_tag or schema_version_changed
        )
        markers, correct_markers = collect_schema_bump_markers(target.commit_range)

        if not schema_changed:
            if markers:
                if args.allow_orphan_footer and len(markers) == 1:
                    marker = markers[0]
                    print(
                        "No schema release validation change needed for the validated range; "
                        "allowing rescue footer "
                        f"{marker.bump} from {marker.commit}."
                    )
                    return 0
                raise ReleasePrepError(
                    "Found Schema-Bump footer(s) but generated schemas and STYLE_SCHEMA_VERSION "
                    f"did not change: {format_marker_list(markers)}"
                )
            print(
                f"No schema release validation changes since {target.baseline_label}; "
                "generated schemas and STYLE_SCHEMA_VERSION are unchanged."
            )
            return 0

        # Schema-Correct: <version> allows an explicit version override — used when
        # correcting an erroneous prior release rather than bumping forward.
        if correct_markers:
            if len(correct_markers) > 1:
                raise ReleasePrepError(
                    "Multiple Schema-Correct footers found across range; only one is allowed."
                )
            correct = correct_markers[0]
            if current_schema_version != correct.version:
                raise ReleasePrepError(
                    f"Schema-Correct footer specifies {correct.version} but "
                    f"STYLE_SCHEMA_VERSION is {current_schema_version}."
                )
            print(
                f"Schema version corrected to {correct.version} via Schema-Correct footer "
                f"from {correct.commit}; skipping forward-bump validation."
            )
            return 0

        if not markers:
            raise ReleasePrepError(
                "Schema changes were detected but no Schema-Bump footer was found "
                f"across {target.commit_range}."
            )

        bump_order = {"patch": 0, "minor": 1, "major": 2}
        marker = max(markers, key=lambda m: bump_order.get(m.bump, -1))
        if len(markers) > 1:
            print(
                f"Multiple Schema-Bump footers found; using highest severity "
                f"({marker.bump} from {marker.commit}): {format_marker_list(markers)}"
            )
        expected_version = bump_version(previous_schema_version, marker.bump)
        if current_schema_version != expected_version:
            raise ReleasePrepError(
                "Schema changes were detected, but STYLE_SCHEMA_VERSION does not match the "
                f"declared Schema-Bump footer. Expected {expected_version}, found "
                f"{current_schema_version}. Commit the schema version bump before release."
            )

        print(
            "Schema release metadata validated: "
            f"STYLE_SCHEMA_VERSION {previous_schema_version} -> {current_schema_version}, "
            f"Schema-Bump footer {marker.bump} from {marker.commit}."
        )
        return 0
    except ReleasePrepError as exc:
        print(f"error: {exc}", flush=True)
        return 1
    except subprocess.CalledProcessError as exc:
        stderr = exc.stderr.strip() if exc.stderr else ""
        stdout = exc.stdout.strip() if exc.stdout else ""
        print(f"error: {stderr or stdout or exc}", flush=True)
        return exc.returncode or 1


if __name__ == "__main__":
    raise SystemExit(main(__import__("sys").argv[1:]))
