#!/usr/bin/env python3
"""Regression tests for release artifact preparation helpers."""

from __future__ import annotations

import argparse
import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT_PATH = Path(__file__).resolve().parent / "prepare-release-artifacts.py"
SPEC = importlib.util.spec_from_file_location("prepare_release_artifacts", SCRIPT_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"Could not load release prep module from {SCRIPT_PATH}")
prep = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = prep
SPEC.loader.exec_module(prep)


class ResolveValidationTargetTests(unittest.TestCase):
    """Verify CLI baseline resolution for release-prep validation."""

    def test_previous_tag_defaults_commit_range(self) -> None:
        args = argparse.Namespace(
            previous_tag="v0.16.0",
            baseline_ref=None,
            commit_range=None,
            dry_run=False,
            allow_orphan_footer=False,
        )

        target = prep.resolve_validation_target(args)

        self.assertEqual(target.baseline_ref, "v0.16.0")
        self.assertEqual(target.baseline_label, "v0.16.0")
        self.assertEqual(target.commit_range, "v0.16.0..HEAD")

    def test_baseline_ref_defaults_commit_range(self) -> None:
        args = argparse.Namespace(
            previous_tag=None,
            baseline_ref="origin/main",
            commit_range=None,
            dry_run=True,
            allow_orphan_footer=False,
        )

        target = prep.resolve_validation_target(args)

        self.assertEqual(target.baseline_ref, "origin/main")
        self.assertEqual(target.baseline_label, "baseline origin/main")
        self.assertEqual(target.commit_range, "origin/main..HEAD")

    def test_explicit_commit_range_overrides_default(self) -> None:
        args = argparse.Namespace(
            previous_tag=None,
            baseline_ref="abc1234",
            commit_range="abc1234..HEAD",
            dry_run=True,
            allow_orphan_footer=True,
        )

        target = prep.resolve_validation_target(args)

        self.assertEqual(target.commit_range, "abc1234..HEAD")


class MarkerFormattingTests(unittest.TestCase):
    """Verify schema bump marker reporting stays readable."""

    def test_format_marker_list_renders_short_summary(self) -> None:
        markers = [
            prep.SchemaBumpMarker(commit="abc12345", subject="feat(schema): one", bump="minor"),
            prep.SchemaBumpMarker(commit="def67890", subject="fix(schema): two", bump="patch"),
        ]

        self.assertEqual(
            prep.format_marker_list(markers),
            "abc12345 (minor), def67890 (patch)",
        )


if __name__ == "__main__":
    unittest.main()
