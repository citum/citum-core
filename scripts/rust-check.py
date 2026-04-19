#!/usr/bin/env python3
"""
Rust workspace version change detection and enforcement for citum-core.

Subcommands:
  pre-commit    - Detect publishable crate changes, infer bump, auto-update workspace version
  commit-msg    - Enforce Version-Bump footer in commit messages
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
from pathlib import Path


CARGO_HOME = os.environ.get("CARGO_HOME") or os.path.expanduser("~/.cargo")
_cargo_candidate = Path(CARGO_HOME) / "bin" / "cargo"
CARGO_BIN = str(_cargo_candidate) if _cargo_candidate.exists() else (shutil.which("cargo") or "cargo")
_semver_candidate = Path(CARGO_HOME) / "bin" / "cargo-semver-checks"
SEMVER_CHECKS_BIN = (
    str(_semver_candidate) if _semver_candidate.exists() else shutil.which("cargo-semver-checks")
)

CARGO_TOML = Path("Cargo.toml")
VERSION_BUMP_RE = re.compile(r"^Version-Bump:\s*(patch|minor|major)\s*$", re.MULTILINE)
WORKSPACE_VERSION_RE = re.compile(
    r"(\[workspace\.package\][^\[]*?version\s*=\s*\")([^\"]+)(\")",
    re.DOTALL,
)

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


def is_publishable_crate_path(path: str) -> bool:
    """Return true when a staged path is in a publishable crate."""
    return path.endswith(".rs") and any(path.startswith(p) for p in PUBLISHABLE_PREFIXES)


def get_staged_files() -> list[str]:
    """Return the currently staged file list."""
    result = subprocess.run(
        ["git", "diff", "--cached", "--name-only"],
        capture_output=True,
        text=True,
    )
    return result.stdout.splitlines()


def get_root_dir() -> Path:
    """Return the git repository root."""
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True,
        text=True,
        cwd=os.getcwd(),
    )
    if result.returncode != 0:
        sys.stderr.write("Error: not in a git repository\n")
        sys.exit(1)
    return Path(result.stdout.strip())


def get_git_dir() -> Path:
    """Return the actual git directory (handles worktrees where .git is a file)."""
    result = subprocess.run(
        ["git", "rev-parse", "--git-dir"],
        capture_output=True,
        text=True,
        cwd=os.getcwd(),
    )
    if result.returncode != 0:
        sys.stderr.write("Error: could not determine git dir\n")
        sys.exit(1)
    return Path(result.stdout.strip())


def read_workspace_version_from_text(content: str) -> str:
    """Extract the workspace version from Cargo.toml content."""
    match = WORKSPACE_VERSION_RE.search(content)
    if match is None:
        raise RuntimeError("Could not determine [workspace.package].version from Cargo.toml")
    return match.group(2)


def read_workspace_version_at_ref(root: Path, ref: str) -> str:
    """Read the workspace version from a git ref."""
    result = subprocess.run(
        ["git", "show", f"{ref}:Cargo.toml"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return read_workspace_version_from_text((root / "Cargo.toml").read_text())
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
            # Pre-1.0: treat breaking changes as minor bumps (matches release-plz behaviour).
            minor += 1
        else:
            major += 1
            minor = 0
        patch = 0
    else:
        raise RuntimeError(f"Unsupported bump type: {bump_type}")
    return f"{major}.{minor}.{patch}"


def update_workspace_version(root: Path, new_version: str) -> None:
    """Write the new workspace version into root Cargo.toml."""
    cargo_toml = root / "Cargo.toml"
    content = cargo_toml.read_text()
    updated, count = WORKSPACE_VERSION_RE.subn(
        rf"\g<1>{new_version}\g<3>", content, count=1
    )
    if count == 0:
        raise RuntimeError("Could not locate [workspace.package].version to update")
    cargo_toml.write_text(updated)


def infer_bump_type_from_diff(staged_files: list[str]) -> str:
    """
    Classify version bump type from staged diff in publishable crates.

    Looks for added/removed public items: removals → major, additions → minor, else → patch.
    Only considers publishable crate paths.
    """
    relevant = [f for f in staged_files if is_publishable_crate_path(f)]
    if not relevant:
        return "patch"

    result = subprocess.run(
        ["git", "diff", "--cached", "--unified=0", "--"] + relevant,
        capture_output=True,
        text=True,
    )
    diff = result.stdout

    removed_pub = [
        line
        for line in diff.splitlines()
        if line.startswith("-") and not line.startswith("---")
        and re.search(r"^\-\s*pub\s+", line)
    ]
    added_pub = [
        line
        for line in diff.splitlines()
        if line.startswith("+") and not line.startswith("+++")
        and re.search(r"^\+\s*pub\s+", line)
    ]

    if removed_pub:
        return "major"
    if added_pub:
        return "minor"
    return "patch"


def run_semver_checks(root: Path) -> str | None:
    """
    Run cargo-semver-checks against HEAD as baseline.

    Returns 'major' if breaking changes found, 'compatible' if not, None on failure/unavailable.
    """
    if SEMVER_CHECKS_BIN is None:
        return None

    try:
        result = subprocess.run(
            [SEMVER_CHECKS_BIN, "--baseline-rev", "HEAD", "--workspace"],
            cwd=root,
            capture_output=True,
            text=True,
            timeout=180,
        )
        if result.returncode == 0:
            return "compatible"
        if result.returncode == 1:
            # Exit code 1 = breaking changes detected.
            return "major"
        # Any other non-zero exit code is a tool execution failure.
        return None
    except (subprocess.TimeoutExpired, FileNotFoundError, OSError):
        return None


def clear_handoff_file() -> None:
    """Remove the version bump handoff file if it exists."""
    handoff = get_git_dir() / "VERSION_BUMP"
    if handoff.exists():
        handoff.unlink()


def pre_commit_hook() -> int:
    """
    pre-commit subcommand: detect publishable crate changes and auto-bump workspace version.
    """
    if os.environ.get("SKIP_VERSION_CHECK"):
        return 0

    root = get_root_dir()
    staged_files = get_staged_files()
    publishable_staged = [f for f in staged_files if is_publishable_crate_path(f)]

    if not publishable_staged:
        clear_handoff_file()
        return 0

    # Check if HEAD exists (skip on first commit).
    head_exists = subprocess.run(
        ["git", "rev-parse", "--verify", "HEAD"],
        capture_output=True,
    ).returncode == 0

    if not head_exists:
        clear_handoff_file()
        return 0

    # Try cargo-semver-checks first; fall back to diff heuristic.
    semver_result = run_semver_checks(root)

    if semver_result == "major":
        bump_type = "major"
    elif semver_result == "compatible":
        # semver-checks confirms no breaking changes; use diff to distinguish minor/patch.
        bump_type = infer_bump_type_from_diff(staged_files)
        if bump_type == "major":
            # Diff saw removed pubs but semver-checks disagrees — trust semver-checks.
            bump_type = "minor"
    else:
        # semver-checks unavailable or failed — use diff heuristic alone.
        bump_type = infer_bump_type_from_diff(staged_files)

    cargo_toml = root / "Cargo.toml"
    current_version = read_workspace_version_from_text(cargo_toml.read_text())
    new_version = bump_version(current_version, bump_type)

    update_workspace_version(root, new_version)
    subprocess.run(["git", "add", "Cargo.toml"], cwd=root)

    # Update Cargo.lock if present by triggering a metadata refresh.
    cargo_lock = root / "Cargo.lock"
    if cargo_lock.exists():
        result = subprocess.run(
            [CARGO_BIN, "update", "--workspace", "--precise", new_version],
            cwd=root,
            capture_output=True,
            timeout=60,
        )
        if result.returncode == 0:
            subprocess.run(["git", "add", "Cargo.lock"], cwd=root)

    handoff = get_git_dir() / "VERSION_BUMP"
    handoff.write_text(bump_type + "\n")

    sys.stdout.write(
        f"[rust-check] Publishable crate changes detected.\n"
        f"[rust-check] Bumped workspace version: {current_version} → {new_version} ({bump_type})\n"
    )

    return 0


def validate_staged_version(root: Path, bump_type: str) -> int:
    """Ensure the staged Cargo.toml version matches the declared bump."""
    cargo_toml = root / "Cargo.toml"
    staged_result = subprocess.run(
        ["git", "show", ":Cargo.toml"],
        cwd=root,
        capture_output=True,
        text=True,
    )
    if staged_result.returncode != 0:
        staged_version = read_workspace_version_from_text(cargo_toml.read_text())
    else:
        staged_version = read_workspace_version_from_text(staged_result.stdout)

    previous_version = read_workspace_version_at_ref(root, "HEAD")
    expected_version = bump_version(previous_version, bump_type)

    if staged_version != expected_version:
        sys.stderr.write(
            f"Error: workspace version does not match Version-Bump: {bump_type}. "
            f"Expected {expected_version}, found {staged_version}.\n"
            f"Manually update [workspace.package].version to {expected_version} in "
            "Cargo.toml and stage the result before committing.\n"
        )
        return 1
    return 0


def commit_msg_hook(msg_file_path: str) -> int:
    """
    commit-msg subcommand: enforce Version-Bump footer when publishable crate changes are staged.
    """
    if os.environ.get("SKIP_VERSION_CHECK"):
        return 0

    root = get_root_dir()
    msg_file = Path(msg_file_path)

    if not msg_file.exists():
        return 0

    msg_content = msg_file.read_text()
    staged_files = get_staged_files()
    cargo_toml_staged = any(f == "Cargo.toml" for f in staged_files)
    handoff = get_git_dir() / "VERSION_BUMP"
    footer_match = VERSION_BUMP_RE.search(msg_content)

    if footer_match:
        if cargo_toml_staged and validate_staged_version(root, footer_match.group(1)) != 0:
            return 1
        if handoff.exists():
            clear_handoff_file()
        return 0

    if handoff.exists():
        if not cargo_toml_staged:
            clear_handoff_file()
            return 0

        bump_type = handoff.read_text().strip()

        if validate_staged_version(root, bump_type) != 0:
            return 1

        if not msg_content.endswith("\n"):
            msg_content += "\n"
        msg_content += f"\nVersion-Bump: {bump_type}\n"
        msg_file.write_text(msg_content)
        handoff.unlink()
        sys.stdout.write(f"[rust-check] Added Version-Bump: {bump_type}\n")

        if bump_type == "major":
            sys.stderr.write(
                "\n" + "=" * 70 + "\n"
                "WARNING: Major version bump detected. Review breaking API changes.\n"
                "=" * 70 + "\n"
            )

        return 0

    if cargo_toml_staged:
        # Cargo.toml staged but no handoff and no footer.
        previous_version = read_workspace_version_at_ref(root, "HEAD")
        current_staged = subprocess.run(
            ["git", "show", ":Cargo.toml"],
            cwd=root,
            capture_output=True,
            text=True,
        )
        if current_staged.returncode == 0:
            staged_version = read_workspace_version_from_text(current_staged.stdout)
            if staged_version != previous_version:
                sys.stderr.write(
                    "Error: Cargo.toml workspace version changed but no Version-Bump footer found.\n"
                    "Add Version-Bump: patch|minor|major to the commit message.\n"
                )
                return 1

    return 0


def main() -> None:
    if len(sys.argv) < 2:
        sys.stderr.write(
            "Usage: rust-check.py pre-commit|commit-msg [commit-msg-file]\n"
        )
        sys.exit(1)

    subcommand = sys.argv[1]

    if subcommand == "pre-commit":
        sys.exit(pre_commit_hook())
    elif subcommand == "commit-msg":
        if len(sys.argv) < 3:
            sys.stderr.write("Usage: rust-check.py commit-msg <msg-file>\n")
            sys.exit(1)
        sys.exit(commit_msg_hook(sys.argv[2]))
    else:
        sys.stderr.write(f"Unknown subcommand: {subcommand}\n")
        sys.exit(1)


if __name__ == "__main__":
    main()
