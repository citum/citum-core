#!/usr/bin/env python3
"""Schema bump workflow for Citum with release-plz-aware guidance."""

from __future__ import annotations

import argparse
import difflib
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import date
from pathlib import Path
from typing import Sequence


REPO_ROOT = Path(__file__).resolve().parent.parent
SCHEMA_LIB = REPO_ROOT / "crates/citum-schema/src/lib.rs"
SCHEMA_DOC = REPO_ROOT / "docs/reference/SCHEMA_VERSIONING.md"
RELEASE_PLZ_WORKFLOW = REPO_ROOT / ".github/workflows/release-plz.yml"
TRACK_CHOICES = ("schema", "code", "engine", "all")
BUMP_CHOICES = ("patch", "minor", "major")

BLUE = "\033[0;34m"
GREEN = "\033[0;32m"
YELLOW = "\033[1;33m"
RED = "\033[0;31m"
CYAN = "\033[0;36m"
NC = "\033[0m"


class BumpError(RuntimeError):
    """Raised when the bump workflow cannot complete safely."""


@dataclass(frozen=True)
class ReleasePlan:
    """Resolved bump details for one invocation."""

    track: str
    bump_type: str
    old_version: str
    new_version: str
    release_name: str
    files_to_modify: tuple[Path, ...]
    tags_to_create: tuple[str, ...]
    commit_message: str
    changelog_tag_prefix: str


@dataclass(frozen=True)
class FileSnapshot:
    """Captured pre-run state for a file the bump may modify."""

    existed: bool
    content: str


def info(message: str) -> None:
    print(f"{BLUE}[INFO]{NC} {message}")


def success(message: str) -> None:
    print(f"{GREEN}[SUCCESS]{NC} {message}")


def warn(message: str) -> None:
    print(f"{YELLOW}[WARN]{NC} {message}")


def error(message: str) -> None:
    print(f"{RED}[ERROR]{NC} {message}", file=sys.stderr)


def header(message: str) -> None:
    print(f"\n{CYAN}-- {message} --{NC}")


def run_git(args: Sequence[str], check: bool = True) -> subprocess.CompletedProcess[str]:
    """Run a git command in the repository root."""

    return subprocess.run(
        ["git", *args],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=check,
    )


def read_schema_version() -> str:
    """Read the default schema version from citum-schema."""

    content = SCHEMA_LIB.read_text(encoding="utf-8")
    match = re.search(r'fn default_version\(\) -> String \{\s*"([^"]+)"\.to_string\(\)\s*\}', content)
    if match is None:
        raise BumpError(
            "Could not find a string-returning default_version() in crates/citum-schema/src/lib.rs"
        )
    return match.group(1)


def bump_version(current: str, bump_type: str) -> str:
    """Return the semantic version produced by the chosen bump type."""

    major_s, minor_s, patch_s = current.split(".")
    major, minor, patch = int(major_s), int(minor_s), int(patch_s)
    if bump_type == "patch":
        patch += 1
    elif bump_type == "minor":
        minor += 1
        patch = 0
    elif bump_type == "major":
        major += 1
        minor = 0
        patch = 0
    else:
        raise BumpError(f"Unsupported bump type: {bump_type}")
    return f"{major}.{minor}.{patch}"


def build_commit_message(track: str, old_version: str, new_version: str, release_name: str) -> str:
    """Build the commit message for the selected track."""

    name_suffix = f" - {release_name}" if release_name else ""
    if track == "schema":
        return (
            f"chore(schema): bump to schema-v{new_version}{name_suffix}\n\n"
            f"Schema {old_version} -> {new_version}."
        )
    raise BumpError(f"Unsupported local bump track: {track}")


def resolve_plan(track: str, bump_type: str, release_name: str) -> ReleasePlan:
    """Resolve the full release plan before any edits occur."""

    old_version = read_schema_version()
    new_version = bump_version(old_version, bump_type)

    if track == "schema":
        files_to_modify = [SCHEMA_LIB, SCHEMA_DOC]
        tags_to_create = (f"schema-v{new_version}",)
        changelog_tag_prefix = "schema-v"
    else:
        raise BumpError(
            "Local code bumps are disabled in this repository. "
            "Code releases are managed by release-plz; use this command only for schema bumps."
        )

    return ReleasePlan(
        track=track,
        bump_type=bump_type,
        old_version=old_version,
        new_version=new_version,
        release_name=release_name,
        files_to_modify=tuple(files_to_modify),
        tags_to_create=tags_to_create,
        commit_message=build_commit_message(track, old_version, new_version, release_name),
        changelog_tag_prefix=changelog_tag_prefix,
    )


def get_dirty_targets(plan: ReleasePlan) -> list[str]:
    """Return targeted tracked files that already have uncommitted changes."""

    result = run_git(["status", "--short", "--", *[str(path.relative_to(REPO_ROOT)) for path in plan.files_to_modify]])
    dirty = []
    for line in result.stdout.splitlines():
        if not line.strip():
            continue
        dirty.append(line[3:])
    return dirty


def ensure_clean_targets(plan: ReleasePlan) -> None:
    """Abort if the files this run would touch are already dirty."""

    dirty = get_dirty_targets(plan)
    if dirty:
        joined = ", ".join(dirty)
        raise BumpError(
            "Refusing to run with uncommitted changes in targeted files: "
            f"{joined}. Commit, stash, or revert them first."
        )


def latest_tag(tag_prefix: str) -> str | None:
    """Return the newest matching tag, if any."""

    result = run_git(["tag", "--sort=version:refname", "-l", f"{tag_prefix}*"])
    tags = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    if not tags:
        return None
    return tags[-1]


def print_changelog(plan: ReleasePlan) -> None:
    """Show commits since the most recent matching tag."""

    tag = latest_tag(plan.changelog_tag_prefix)
    if tag is None:
        info(f"No previous {plan.changelog_tag_prefix}* tags; recent commits:")
        result = run_git(["--no-pager", "log", "--oneline", "-20"])
    else:
        info(f"Changes since {tag}:")
        result = run_git(["--no-pager", "log", f"{tag}..HEAD", "--oneline"])
    output = result.stdout.rstrip()
    if output:
        print(output)
    else:
        warn("No commits found for this release range.")


def print_preview(plan: ReleasePlan) -> None:
    """Print a precise summary of what the bump will do."""

    if plan.track == "schema":
        header(f"Schema release bump: {plan.old_version} -> {plan.new_version}")
        print("  Scope        : bump the default style schema version without changing code release versions")
        print(f"  Bump type    : {plan.bump_type}")
        print(f"  Schema lib   : update default_version() in {SCHEMA_LIB.relative_to(REPO_ROOT)}")
        print(f"  Schema tag   : {plan.tags_to_create[0]}")
        print(f"  Schema doc   : add changelog entry in {SCHEMA_DOC.relative_to(REPO_ROOT)}")
    else:
        raise BumpError(f"Unsupported local bump track: {plan.track}")

    if plan.release_name:
        print(f"  Release name : {plan.release_name}")

    print("  Files        :")
    for path in plan.files_to_modify:
        print(f"    - {path.relative_to(REPO_ROOT)}")
    print()
    print_changelog(plan)
    print()


def confirm_prompt() -> bool:
    """Ask the user whether to proceed."""

    reply = input("Proceed with this version bump? (y/N) ").strip()
    return reply.lower() == "y"


def update_schema_version(plan: ReleasePlan) -> None:
    """Replace the default schema version in citum-schema."""

    content = SCHEMA_LIB.read_text(encoding="utf-8")
    pattern = r'(fn default_version\(\) -> String \{\s*")([^"]+)("\.to_string\(\)\s*\})'
    updated, count = re.subn(pattern, rf"\g<1>{plan.new_version}\g<3>", content, count=1)
    if count != 1:
        raise BumpError("Failed to update default_version() in crates/citum-schema/src/lib.rs")
    SCHEMA_LIB.write_text(updated, encoding="utf-8")


def update_schema_doc(plan: ReleasePlan) -> None:
    """Insert a schema changelog entry at the top of the changelog section."""

    content = SCHEMA_DOC.read_text(encoding="utf-8")
    marker = "### Schema Changelog\n\nTrack schema changes separately from code changes:\n\n"
    if marker not in content:
        raise BumpError("Could not find the schema changelog section in SCHEMA_VERSIONING.md")
    name_suffix = f" - {plan.release_name}" if plan.release_name else ""
    entry = (
        f"#### schema-v{plan.new_version} ({date.today().isoformat()}){name_suffix}\n"
        f"- Schema version bumped from {plan.old_version} to {plan.new_version}\n\n"
    )
    SCHEMA_DOC.write_text(content.replace(marker, marker + entry, 1), encoding="utf-8")


def validate_build() -> None:
    """Run the existing validation step after applying the bump."""

    info("Validating with cargo test --quiet --lib")
    result = subprocess.run(
        ["cargo", "test", "--quiet", "--lib"],
        cwd=REPO_ROOT,
        text=True,
    )
    if result.returncode != 0:
        raise BumpError("cargo test --quiet --lib failed")
    success("Build validation passed")


def print_diff(path: Path, original: str, updated: str) -> None:
    """Print a unified diff for one modified file."""

    if original == updated:
        return
    relative = path.relative_to(REPO_ROOT)
    diff = difflib.unified_diff(
        original.splitlines(),
        updated.splitlines(),
        fromfile=f"a/{relative}",
        tofile=f"b/{relative}",
        lineterm="",
    )
    print("\n".join(diff))


def show_review(modified_files: dict[Path, FileSnapshot]) -> None:
    """Print diffs for all files changed by this run."""

    print()
    info("Review changes:")
    for path, snapshot in modified_files.items():
        if not path.exists():
            continue
        print_diff(path, snapshot.content, path.read_text(encoding="utf-8"))


def stage_commit_and_tag(plan: ReleasePlan) -> None:
    """Stage, commit, and tag the version bump."""

    run_git(["add", "--", *[str(path.relative_to(REPO_ROOT)) for path in plan.files_to_modify]])
    run_git(["commit", "-m", plan.commit_message])
    success("Changes committed")

    tag_message_suffix = f" - {plan.release_name}" if plan.release_name else ""
    for tag in plan.tags_to_create:
        info(f"Creating tag: {tag}")
        run_git(["tag", "-a", tag, "-m", f"Version {tag}{tag_message_suffix}"])
        success(f"Tag created: {tag}")


def capture_snapshots(paths: Sequence[Path]) -> dict[Path, FileSnapshot]:
    """Capture whether each file exists and, if so, its contents."""

    snapshots: dict[Path, FileSnapshot] = {}
    for path in paths:
        if path.exists():
            snapshots[path] = FileSnapshot(existed=True, content=path.read_text(encoding="utf-8"))
        else:
            snapshots[path] = FileSnapshot(existed=False, content="")
    return snapshots


def rollback(modified_files: dict[Path, FileSnapshot]) -> None:
    """Restore original file contents for files changed during this run."""

    for path, snapshot in modified_files.items():
        if snapshot.existed:
            path.write_text(snapshot.content, encoding="utf-8")
        elif path.exists():
            path.unlink()


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    """Parse command-line arguments, including the legacy shorthand form."""

    normalized = list(argv)
    if normalized and normalized[0] in BUMP_CHOICES:
        normalized.insert(0, "code")

    parser = argparse.ArgumentParser(
        prog="./scripts/bump.sh",
        description="Bump Citum engine/schema release versions.",
    )
    parser.add_argument("track", choices=TRACK_CHOICES, help="Release track to bump.")
    parser.add_argument("bump_type", choices=BUMP_CHOICES, help="Semantic version bump type.")
    parser.add_argument("--dry-run", action="store_true", help="Preview actions without modifying files.")
    parser.add_argument("--name", default="", help="Optional release title used in commit/tag text.")
    return parser.parse_args(normalized)


def main(argv: Sequence[str]) -> int:
    """Run the release bump workflow."""

    try:
        args = parse_args(argv)
        if args.track != "schema" and RELEASE_PLZ_WORKFLOW.exists():
            raise BumpError(
                "Code releases are managed by release-plz in this repository. "
                "Use `./scripts/bump.sh schema <patch|minor|major>` for schema-only bumps. "
                "Do not use this command to bump Cargo versions or create `v*` tags."
            )
        plan = resolve_plan(args.track, args.bump_type, args.name)
        print_preview(plan)

        if args.dry_run:
            success("Dry-run complete; no files modified")
            return 0

        ensure_clean_targets(plan)

        if not confirm_prompt():
            info("Cancelled before making changes")
            return 0

        modified_files = capture_snapshots(plan.files_to_modify)
        try:
            update_schema_version(plan)
            update_schema_doc(plan)
            validate_build()
            show_review(modified_files)
            stage_commit_and_tag(plan)
        except Exception:
            rollback(modified_files)
            raise

        print()
        success("Version bump complete!")
        print("  Push: git push && git push --tags")
        return 0
    except BumpError as exc:
        error(str(exc))
        return 1
    except KeyboardInterrupt:
        warn("Interrupted")
        return 130
    except subprocess.CalledProcessError as exc:
        stderr = exc.stderr.strip() if exc.stderr else ""
        error(stderr or str(exc))
        return exc.returncode or 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
