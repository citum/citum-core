#!/usr/bin/env python3
"""Regression tests for release workflow invariants."""

from __future__ import annotations

import re
import subprocess
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
WORKFLOW_PATH = REPO_ROOT / ".github/workflows/release.yml"
RELEASE_CONFIG_PATH = REPO_ROOT / "release.toml"
SCHEMA_LIB = REPO_ROOT / "crates/citum-schema-style/src/lib.rs"


class ReleaseWorkflowTests(unittest.TestCase):
    """Ensure release workflow semantics stay aligned with policy."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
        cls.release_config = RELEASE_CONFIG_PATH.read_text(encoding="utf-8")

    def test_release_branch_is_always_release_next(self) -> None:
        self.assertIn('echo "branch=release/next" >> "$GITHUB_OUTPUT"', self.workflow)
        self.assertNotIn('echo "branch=main" >> "$GITHUB_OUTPUT"', self.workflow)

    def test_release_pr_is_always_enabled(self) -> None:
        self.assertIn('echo "create_pr=true" >> "$GITHUB_OUTPUT"', self.workflow)
        self.assertNotIn('echo "create_pr=false" >> "$GITHUB_OUTPUT"', self.workflow)

    def test_cargo_release_does_not_use_metadata_flag_for_commit_message(self) -> None:
        bump_workspace_block = re.search(
            r"- name: Bump workspace version.*?cargo release .*?\n",
            self.workflow,
            flags=re.DOTALL,
        )
        self.assertIsNotNone(bump_workspace_block)
        assert bump_workspace_block is not None
        self.assertNotIn(" -m ", bump_workspace_block.group(0))

    def test_release_hook_writes_repo_root_changelog(self) -> None:
        self.assertNotIn('"git-cliff", "-o", "CHANGELOG.md"', self.release_config)
        self.assertIn("git rev-parse --show-toplevel", self.release_config)
        self.assertIn("$repo/CHANGELOG.md", self.release_config)

    def test_crate_changelogs_are_absent(self) -> None:
        tracked = subprocess.run(
            ["git", "ls-files", "crates/*/CHANGELOG.md"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=True,
        )
        existing_tracked = [
            path
            for path in tracked.stdout.splitlines()
            if (REPO_ROOT / path).exists()
        ]

        self.assertEqual(existing_tracked, [])

    def test_schema_tag_steps_do_not_use_heredoc(self) -> None:
        """Heredoc closing delimiters can gain indentation via YAML processing,
        causing 'unexpected EOF' shell errors. Both schema-tag steps must use
        a plain command instead of a Python heredoc to extract the version."""
        schema_tag_blocks = re.findall(
            r"- name: Tag schema release when schema changed.*?(?=\n      -|\Z)",
            self.workflow,
            flags=re.DOTALL,
        )
        self.assertTrue(schema_tag_blocks, "Expected at least one 'Tag schema release' step")
        for block in schema_tag_blocks:
            self.assertNotIn("<<'PY'", block, "Python heredoc found in schema tag step")
            self.assertNotIn("python3 -", block, "Python heredoc invocation found in schema tag step")
            self.assertIn("STYLE_SCHEMA_VERSION", block, "Step must reference STYLE_SCHEMA_VERSION")

    def test_schema_version_sed_pattern_extracts_correctly(self) -> None:
        """The sed command used in auto-tag must extract the correct version
        from the actual lib.rs file on disk."""
        result = subprocess.run(
            [
                "sed",
                "-n",
                r"s/.*STYLE_SCHEMA_VERSION: \&str = \"\([^\"]*\)\".*/\1/p",
                str(SCHEMA_LIB),
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        version = result.stdout.strip()
        self.assertRegex(
            version,
            r"^\d+\.\d+\.\d+$",
            f"sed did not extract a valid semver from {SCHEMA_LIB}; got: {version!r}",
        )


if __name__ == "__main__":
    unittest.main()
