#!/usr/bin/env python3
"""
Schema change detection and enforcement for citum-core.

Subcommands:
  pre-commit    - Check if schemas need regeneration and auto-update docs/schemas/
  commit-msg    - Enforce Schema-Bump footer in commit messages
"""

import os
import sys
import subprocess
import tempfile
from pathlib import Path
import filecmp
import difflib


def get_root_dir():
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


def schema_files_staged():
    """Check if any citum-schema* files are staged."""
    result = subprocess.run(
        ["git", "diff", "--cached", "--name-only"],
        capture_output=True,
        text=True,
    )
    staged_files = result.stdout.splitlines()
    for f in staged_files:
        if "crates/citum-schema" in f and f.endswith(".rs"):
            return True
    return False


def infer_bump_type(old_dir: Path, new_dir: Path) -> str:
    """
    Infer schema version bump type by comparing old and new schemas.

    Major: if any properties/fields are removed or type changes detected
    Patch: if only additions or non-breaking changes
    """
    result = subprocess.run(
        ["diff", "-ru", str(old_dir), str(new_dir)],
        capture_output=True,
        text=True,
    )
    removal_lines = [
        line
        for line in result.stdout.splitlines()
        if line.startswith("-") and not line.startswith("---")
    ]
    # Any removals = major; additions only = patch
    return "major" if removal_lines else "patch"


def regenerate_schemas(root: Path, tmpdir: Path) -> bool:
    """
    Run cargo to regenerate schemas into tmpdir.
    Returns True if successful, False otherwise.
    """
    try:
        subprocess.run(
            [
                "cargo",
                "run",
                "--bin",
                "citum",
                "--features",
                "schema",
                "--quiet",
                "--",
                "schema",
                "--out-dir",
                str(tmpdir),
            ],
            cwd=root,
            capture_output=True,
            timeout=120,
            check=True,
        )
        return True
    except subprocess.CalledProcessError:
        return False
    except subprocess.TimeoutExpired:
        return False


def schemas_differ(old_dir: Path, new_dir: Path) -> bool:
    """Check if schemas in old_dir and new_dir differ."""
    # Compare recursively
    cmp = filecmp.dircmp(str(old_dir), str(new_dir), ignore=[])
    return bool(cmp.left_only or cmp.right_only or cmp.diff_files)


def pre_commit_hook():
    """
    pre-commit subcommand: check and auto-update schemas if needed.
    """
    if os.environ.get("SKIP_SCHEMA_CHECK"):
        return 0

    root = get_root_dir()

    # Only run if schema files are staged
    if not schema_files_staged():
        return 0

    schemas_dir = root / "docs" / "schemas"
    if not schemas_dir.exists():
        sys.stderr.write(
            "[schema-check] Warning: docs/schemas/ not found; skipping\n"
        )
        return 0

    # Generate into temp directory
    with tempfile.TemporaryDirectory() as tmpdir:
        tmpdir_path = Path(tmpdir)
        if not regenerate_schemas(root, tmpdir_path):
            sys.stderr.write(
                "[schema-check] Warning: cargo schema generation failed; continuing\n"
            )
            return 0

        # Check for changes
        if not schemas_differ(schemas_dir, tmpdir_path):
            sys.stdout.write("[schema-check] Schemas up to date.\n")
            return 0

        # Infer bump type BEFORE overwriting docs/schemas/
        bump_type = infer_bump_type(schemas_dir, tmpdir_path)

        # Update docs/schemas/
        import shutil

        for schema_file in tmpdir_path.glob("*.json"):
            dest = schemas_dir / schema_file.name
            shutil.copy2(schema_file, dest)

        # Stage updated schemas
        subprocess.run(
            ["git", "add", "docs/schemas/"],
            cwd=root,
        )

        # Write to handoff file
        handoff_file = root / ".git" / "SCHEMA_BUMP"
        handoff_file.write_text(bump_type + "\n")

        sys.stdout.write(
            f"[schema-check] Updated docs/schemas/. Inferred bump: {bump_type}\n"
        )

    return 0


def commit_msg_hook(msg_file_path):
    """
    commit-msg subcommand: enforce Schema-Bump footer.
    """
    if os.environ.get("SKIP_SCHEMA_CHECK"):
        return 0

    root = get_root_dir()
    msg_file = Path(msg_file_path)

    if not msg_file.exists():
        return 0

    msg_content = msg_file.read_text()

    # Check if Schema-Bump footer already exists
    if "Schema-Bump:" in msg_content:
        return 0

    # Check for handoff file from pre-commit
    handoff_file = root / ".git" / "SCHEMA_BUMP"
    if handoff_file.exists():
        bump_type = handoff_file.read_text().strip()
        handoff_file.unlink()

        # Append footer to message
        if not msg_content.endswith("\n"):
            msg_content += "\n"
        msg_content += f"\nSchema-Bump: {bump_type}\n"

        msg_file.write_text(msg_content)
        sys.stdout.write(f"[schema-check] Added Schema-Bump: {bump_type}\n")

        if bump_type == "major":
            sys.stderr.write(
                "\n" + "=" * 70 + "\n"
                "WARNING: Major schema bump detected. Review breaking changes.\n"
                "=" * 70 + "\n"
            )

        return 0

    # Check if docs/schemas/ is staged but no footer exists
    result = subprocess.run(
        ["git", "diff", "--cached", "--name-only"],
        capture_output=True,
        text=True,
    )
    staged_files = result.stdout.splitlines()
    schema_files_staged = any(f.startswith("docs/schemas/") for f in staged_files)

    if schema_files_staged:
        sys.stderr.write(
            "Error: docs/schemas/ staged but no Schema-Bump footer.\n"
            "Add Schema-Bump: patch|minor|major to commit message.\n"
        )
        return 1

    return 0


def main():
    if len(sys.argv) < 2:
        sys.stderr.write(
            "Usage: schema-check.py pre-commit|commit-msg [commit-msg-file]\n"
        )
        sys.exit(1)

    subcommand = sys.argv[1]

    if subcommand == "pre-commit":
        sys.exit(pre_commit_hook())
    elif subcommand == "commit-msg":
        if len(sys.argv) < 3:
            sys.stderr.write("Usage: schema-check.py commit-msg <msg-file>\n")
            sys.exit(1)
        sys.exit(commit_msg_hook(sys.argv[2]))
    else:
        sys.stderr.write(f"Unknown subcommand: {subcommand}\n")
        sys.exit(1)


if __name__ == "__main__":
    main()
