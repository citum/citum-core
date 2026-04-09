#!/usr/bin/env python3
"""Regression tests for coverage-analysis note parsing."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT_PATH = Path(__file__).resolve().parent / "coverage-analysis.py"
SPEC = importlib.util.spec_from_file_location("coverage_analysis", SCRIPT_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"Could not load coverage-analysis module from {SCRIPT_PATH}")
coverage = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = coverage
SPEC.loader.exec_module(coverage)


class ExtraVariableParsingTests(unittest.TestCase):
    """Verify note-field override parsing keeps supported variables visible."""

    def test_extract_variables_keeps_underscore_extra_variables(self) -> None:
        items = [
            {
                "id": "item-1",
                "note": "archive_collection: Box 4\narchive_location: Shelf A",
            }
        ]

        analysis = coverage.extract_variables(items)

        self.assertEqual(analysis["extra_vars"]["archive_collection"], 1)
        self.assertEqual(analysis["extra_vars"]["archive_location"], 1)


if __name__ == "__main__":
    unittest.main()
