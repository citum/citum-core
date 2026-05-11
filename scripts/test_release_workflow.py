#!/usr/bin/env python3
"""Regression tests for release workflow invariants."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


WORKFLOW_PATH = Path(__file__).resolve().parent.parent / ".github/workflows/release.yml"


class ReleaseWorkflowTests(unittest.TestCase):
    """Ensure release workflow semantics stay aligned with policy."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")

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


if __name__ == "__main__":
    unittest.main()
