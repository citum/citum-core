#!/usr/bin/env python3
"""Regression tests for squash-merge pull request title validation."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).resolve().parent / "validate-pr-title.py"
SPEC = importlib.util.spec_from_file_location("validate_pr_title", SCRIPT_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"Could not load pull request title validator from {SCRIPT_PATH}")
validator = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = validator
SPEC.loader.exec_module(validator)


class PullRequestTitleTests(unittest.TestCase):
    """Verify GitHub squash-merge titles obey the commit subject policy."""

    def test_rejects_the_historical_sixty_nine_character_title(self) -> None:
        title = "fix(engine): restore locale type-variant fallback in bilingual styles"

        self.assertEqual(
            validator.validate_title(title),
            "pull request title is 69 chars; max allowed is 50",
        )

    def test_accepts_a_fifty_character_title(self) -> None:
        title = "fix(engine): " + "a" * 37

        self.assertEqual(len(title), 50)
        self.assertIsNone(validator.validate_title(title))

    def test_rejects_a_fifty_one_character_title(self) -> None:
        title = "fix(engine): " + "a" * 38

        self.assertEqual(len(title), 51)
        self.assertEqual(
            validator.validate_title(title),
            "pull request title is 51 chars; max allowed is 50",
        )

    def test_rejects_a_scope_outside_the_alint_policy(self) -> None:
        title = "fix(unknown): retain compatibility"

        self.assertEqual(
            validator.validate_title(title),
            "pull request title does not match the conventional-commit policy",
        )


if __name__ == "__main__":
    unittest.main()
