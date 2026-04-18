#!/usr/bin/env python3
"""Regression tests for the schema hook guardrails."""

from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from contextlib import contextmanager
from pathlib import Path

SCRIPT_PATH = Path(__file__).resolve().parent / "schema-check.py"
SPEC = importlib.util.spec_from_file_location("schema_check", SCRIPT_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"Could not load schema hook module from {SCRIPT_PATH}")
schema_check = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = schema_check
SPEC.loader.exec_module(schema_check)


@contextmanager
def pushd(path: Path):
    """Temporarily change the working directory."""

    previous = Path.cwd()
    os.chdir(path)
    try:
        yield
    finally:
        os.chdir(previous)


def run_git(repo: Path, *args: str) -> subprocess.CompletedProcess[str]:
    """Run a git command in the temporary repository."""

    return subprocess.run(
        ["git", *args],
        cwd=repo,
        text=True,
        capture_output=True,
        check=True,
    )


class SchemaCheckHookTests(unittest.TestCase):
    """Verify schema hook behavior stays aligned with CI expectations."""

    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.repo = Path(self.tempdir.name)
        run_git(self.repo, "init", "-q")
        run_git(self.repo, "config", "user.name", "Test User")
        run_git(self.repo, "config", "user.email", "test@example.com")

        (self.repo / "crates/citum-schema-style/src").mkdir(parents=True)
        (self.repo / "docs/schemas").mkdir(parents=True)
        (self.repo / "crates/citum-schema-style/src/lib.rs").write_text(
            'pub const STYLE_SCHEMA_VERSION: &str = "0.16.0";\n',
            encoding="utf-8",
        )
        (self.repo / "docs/schemas/locale.json").write_text('{"version": 1}\n', encoding="utf-8")

        run_git(self.repo, "add", ".")
        run_git(
            self.repo,
            "commit",
            "-m",
            "chore(schema): seed repo\n\nInitial schema baseline.",
        )

    def tearDown(self) -> None:
        self.tempdir.cleanup()

    def write_commit_message(self, body: str) -> Path:
        """Write a temporary commit message file inside the repo."""

        path = self.repo / "COMMIT_EDITMSG"
        path.write_text(textwrap.dedent(body), encoding="utf-8")
        return path

    def stage_schema_change(self, *, bump_version: bool) -> None:
        """Stage a schema artifact change, optionally with the matching version bump."""

        (self.repo / "docs/schemas/locale.json").write_text('{"version": 2}\n', encoding="utf-8")
        run_git(self.repo, "add", "docs/schemas/locale.json")
        if bump_version:
            (self.repo / "crates/citum-schema-style/src/lib.rs").write_text(
                'pub const STYLE_SCHEMA_VERSION: &str = "0.16.1";\n',
                encoding="utf-8",
            )
            run_git(self.repo, "add", "crates/citum-schema-style/src/lib.rs")
        (self.repo / ".git/SCHEMA_BUMP").write_text("patch\n", encoding="utf-8")

    def test_commit_msg_rejects_missing_version_bump(self) -> None:
        self.stage_schema_change(bump_version=False)
        msg_file = self.write_commit_message(
            """
            feat(locale): update schema

            Regenerate schema output.
            """
        )

        with pushd(self.repo):
            result = schema_check.commit_msg_hook(str(msg_file))

        self.assertEqual(result, 1)
        self.assertTrue((self.repo / ".git/SCHEMA_BUMP").exists())
        self.assertNotIn("Schema-Bump:", msg_file.read_text(encoding="utf-8"))

    def test_commit_msg_appends_footer_when_version_matches(self) -> None:
        self.stage_schema_change(bump_version=True)
        msg_file = self.write_commit_message(
            """
            feat(locale): update schema

            Regenerate schema output.
            """
        )

        with pushd(self.repo):
            result = schema_check.commit_msg_hook(str(msg_file))

        self.assertEqual(result, 0)
        self.assertFalse((self.repo / ".git/SCHEMA_BUMP").exists())
        self.assertIn("Schema-Bump: patch", msg_file.read_text(encoding="utf-8"))

    def test_commit_msg_rejects_existing_footer_without_matching_version_bump(self) -> None:
        self.stage_schema_change(bump_version=False)
        msg_file = self.write_commit_message(
            """
            feat(locale): update schema

            Regenerate schema output.

            Schema-Bump: patch
            """
        )

        with pushd(self.repo):
            result = schema_check.commit_msg_hook(str(msg_file))

        self.assertEqual(result, 1)

    def test_commit_msg_existing_footer_clears_stale_handoff_when_version_matches(self) -> None:
        self.stage_schema_change(bump_version=True)
        msg_file = self.write_commit_message(
            """
            feat(locale): update schema

            Regenerate schema output.

            Schema-Bump: patch
            """
        )

        with pushd(self.repo):
            result = schema_check.commit_msg_hook(str(msg_file))

        self.assertEqual(result, 0)
        self.assertFalse((self.repo / ".git/SCHEMA_BUMP").exists())

    def test_commit_msg_existing_footer_keeps_handoff_on_validation_failure(self) -> None:
        self.stage_schema_change(bump_version=False)
        msg_file = self.write_commit_message(
            """
            feat(locale): update schema

            Regenerate schema output.

            Schema-Bump: patch
            """
        )

        with pushd(self.repo):
            result = schema_check.commit_msg_hook(str(msg_file))

        self.assertEqual(result, 1)
        self.assertTrue((self.repo / ".git/SCHEMA_BUMP").exists())

    def test_schema_files_staged_detects_cli_schema_inputs(self) -> None:
        (self.repo / "crates/citum-cli/src").mkdir(parents=True)
        (self.repo / "crates/citum-cli/src/main.rs").write_text(
            "fn main() {}\n",
            encoding="utf-8",
        )
        run_git(self.repo, "add", "crates/citum-cli/src/main.rs")

        with pushd(self.repo):
            self.assertTrue(schema_check.schema_files_staged())


if __name__ == "__main__":
    unittest.main()
