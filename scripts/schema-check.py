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
import re
import shutil


# Resolve cargo binary — git hooks don't inherit the full user PATH.
# Prefer $CARGO_HOME/bin/cargo, then ~/.cargo/bin/cargo, then PATH lookup.
def _find_cargo() -> str:
    cargo_home = os.environ.get("CARGO_HOME") or os.path.expanduser("~/.cargo")
    candidate = Path(cargo_home) / "bin" / "cargo"
    if candidate.exists():
        return str(candidate)
    found = shutil.which("cargo")
    return found if found else "cargo"


CARGO_BIN = _find_cargo()

STYLE_SCHEMA_LIB = Path("crates/citum-schema-style/src/lib.rs")
SCHEMA_BUMP_RE = re.compile(r"^Schema-Bump:\s*(patch|minor|major)\s*$", re.MULTILINE)


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


def schema_files_staged():
    """Check if any citum-schema* files are staged."""
    staged_files = get_staged_files()
    for f in staged_files:
        if "crates/citum-schema" in f and f.endswith(".rs"):
            return True
    return False


def get_staged_files() -> list[str]:
    """Return the currently staged file list."""
    result = subprocess.run(
        ["git", "diff", "--cached", "--name-only"],
        capture_output=True,
        text=True,
    )
    return result.stdout.splitlines()


def schema_artifacts_staged(staged_files: list[str]) -> bool:
    """Return true when staged files include schema artifacts or version metadata."""
    schema_lib = STYLE_SCHEMA_LIB.as_posix()
    return any(
        path.startswith("docs/schemas/") or path == schema_lib for path in staged_files
    )


def clear_handoff_file(root: Path) -> None:
    """Remove the schema bump handoff file if it exists."""
    handoff_file = get_git_dir() / "SCHEMA_BUMP"
    if handoff_file.exists():
        handoff_file.unlink()


def read_schema_version_from_text(content: str) -> str:
    """Extract STYLE_SCHEMA_VERSION from file content."""
    match = re.search(
        r'pub const STYLE_SCHEMA_VERSION: &str = "([^"]+)";',
        content,
    )
    if match is None:
        raise RuntimeError("Could not determine STYLE_SCHEMA_VERSION")
    return match.group(1)


def read_schema_version_from_index(root: Path) -> str:
    """Read the staged STYLE_SCHEMA_VERSION from the git index."""
    result = subprocess.run(
        ["git", "show", f":{STYLE_SCHEMA_LIB.as_posix()}"],
        cwd=root,
        capture_output=True,
        text=True,
        check=True,
    )
    return read_schema_version_from_text(result.stdout)


def read_schema_version_at_ref(root: Path, ref: str) -> str:
    """Read STYLE_SCHEMA_VERSION from a git ref."""
    result = subprocess.run(
        ["git", "show", f"{ref}:{STYLE_SCHEMA_LIB.as_posix()}"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return read_schema_version_from_index(root)
    return read_schema_version_from_text(result.stdout)


def bump_version(current: str, bump_type: str) -> str:
    """Return the semantic version after applying the bump type."""
    major_s, minor_s, patch_s = current.split(".")
    major, minor, patch = int(major_s), int(minor_s), int(patch_s)
    if bump_type == "patch":
        patch += 1
    elif bump_type == "minor":
        minor += 1
        patch = 0
    elif bump_type == "major":
        if major == 0:
            minor += 1
        else:
            major += 1
            minor = 0
        patch = 0
    else:
        raise RuntimeError(f"Unsupported schema bump type: {bump_type}")
    return f"{major}.{minor}.{patch}"


def validate_staged_schema_version(root: Path, bump_type: str) -> int:
    """Ensure the staged STYLE_SCHEMA_VERSION matches the declared bump."""
    previous_version = read_schema_version_at_ref(root, "HEAD")
    staged_version = read_schema_version_from_index(root)
    expected_version = bump_version(previous_version, bump_type)

    if staged_version != expected_version:
        sys.stderr.write(
            "Error: schema changes were staged, but STYLE_SCHEMA_VERSION does not "
            f"match Schema-Bump: {bump_type}. Expected {expected_version}, found "
            f"{staged_version}.\nRun ./scripts/bump.sh schema {bump_type} and "
            "stage the result before committing.\n"
        )
        return 1
    return 0


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
        # description-only changes are never breaking (purely informational)
        and not line.lstrip("- \t").startswith('"description":')
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
                CARGO_BIN,
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
        clear_handoff_file(root)
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
            clear_handoff_file(root)
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
        handoff_file = get_git_dir() / "SCHEMA_BUMP"
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
    staged_files = get_staged_files()
    schema_change_staged = schema_artifacts_staged(staged_files)
    handoff_file = get_git_dir() / "SCHEMA_BUMP"
    footer_match = SCHEMA_BUMP_RE.search(msg_content)

    if footer_match:
        if schema_change_staged and validate_staged_schema_version(root, footer_match.group(1)) != 0:
            return 1
        if handoff_file.exists():
            clear_handoff_file(root)
        return 0

    if handoff_file.exists():
        if not schema_change_staged:
            clear_handoff_file(root)
            return 0

        bump_type = handoff_file.read_text().strip()

        if validate_staged_schema_version(root, bump_type) != 0:
            return 1

        # Append footer to message
        if not msg_content.endswith("\n"):
            msg_content += "\n"
        msg_content += f"\nSchema-Bump: {bump_type}\n"

        msg_file.write_text(msg_content)
        handoff_file.unlink()
        sys.stdout.write(f"[schema-check] Added Schema-Bump: {bump_type}\n")

        if bump_type == "major":
            sys.stderr.write(
                "\n" + "=" * 70 + "\n"
                "WARNING: Major schema bump detected. Review breaking changes.\n"
                "=" * 70 + "\n"
            )

        return 0

    if any(f.startswith("docs/schemas/") for f in staged_files):
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
