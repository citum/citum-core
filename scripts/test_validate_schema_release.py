#!/usr/bin/env python3
"""Regression tests for schema release validation helpers."""

from __future__ import annotations

import argparse
import importlib.util
import sys
import unittest
from pathlib import Path
from unittest import mock

SCRIPT_PATH = Path(__file__).resolve().parent / "validate-schema-release.py"
SPEC = importlib.util.spec_from_file_location("validate_schema_release", SCRIPT_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"Could not load schema validation module from {SCRIPT_PATH}")
prep = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = prep
SPEC.loader.exec_module(prep)


class ResolveValidationTargetTests(unittest.TestCase):
    """Verify CLI baseline resolution for schema release validation."""

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


class BumpVersionTests(unittest.TestCase):
    """Verify schema bump semantics stay aligned with policy."""

    def test_major_bump_on_pre_one_zero_advances_minor(self) -> None:
        self.assertEqual(prep.bump_version("0.13.0", "major"), "0.14.0")

    def test_major_bump_on_one_zero_advances_major(self) -> None:
        self.assertEqual(prep.bump_version("1.2.3", "major"), "2.0.0")


class ValidationOnlyTests(unittest.TestCase):
    """Verify schema validation does not mutate tracked files."""

    @mock.patch.object(prep, "collect_schema_bump_markers")
    @mock.patch.object(prep, "read_schema_dir_contents")
    @mock.patch.object(prep, "export_schemas")
    @mock.patch.object(prep, "read_schema_dir_contents_at_ref")
    @mock.patch.object(prep, "read_current_schema_version")
    @mock.patch.object(prep, "read_schema_version_at_ref")
    @mock.patch.object(prep, "resolve_validation_target")
    @mock.patch.object(prep, "parse_args")
    def test_main_validates_changed_schema_without_mutation(
        self,
        parse_args: mock.Mock,
        resolve_validation_target: mock.Mock,
        read_schema_version_at_ref: mock.Mock,
        read_current_schema_version: mock.Mock,
        read_schema_dir_contents_at_ref: mock.Mock,
        export_schemas: mock.Mock,
        read_schema_dir_contents: mock.Mock,
        collect_schema_bump_markers: mock.Mock,
    ) -> None:
        parse_args.return_value = argparse.Namespace(
            previous_tag="v0.19.0",
            baseline_ref=None,
            commit_range=None,
            dry_run=False,
            allow_orphan_footer=False,
        )
        resolve_validation_target.return_value = prep.ValidationTarget(
            baseline_ref="v0.19.0",
            baseline_label="v0.19.0",
            commit_range="v0.19.0..HEAD",
        )
        read_schema_version_at_ref.return_value = "0.13.0"
        read_current_schema_version.return_value = "0.14.0"
        read_schema_dir_contents_at_ref.return_value = {"style.json": "before"}
        read_schema_dir_contents.side_effect = [
            {"style.json": "after"},
            {"style.json": "after"},
        ]
        collect_schema_bump_markers.return_value = (
            [prep.SchemaBumpMarker(commit="abc12345", subject="feat(schema): split", bump="major")],
            [],
        )

        result = prep.main([])

        self.assertEqual(result, 0)
        export_schemas.assert_called_once()

    @mock.patch.object(prep, "collect_schema_bump_markers")
    @mock.patch.object(prep, "read_schema_dir_contents")
    @mock.patch.object(prep, "export_schemas")
    @mock.patch.object(prep, "read_schema_dir_contents_at_ref")
    @mock.patch.object(prep, "read_current_schema_version")
    @mock.patch.object(prep, "read_schema_version_at_ref")
    @mock.patch.object(prep, "resolve_validation_target")
    @mock.patch.object(prep, "parse_args")
    def test_main_fails_when_committed_schemas_are_stale(
        self,
        parse_args: mock.Mock,
        resolve_validation_target: mock.Mock,
        read_schema_version_at_ref: mock.Mock,
        read_current_schema_version: mock.Mock,
        read_schema_dir_contents_at_ref: mock.Mock,
        export_schemas: mock.Mock,
        read_schema_dir_contents: mock.Mock,
        collect_schema_bump_markers: mock.Mock,
    ) -> None:
        parse_args.return_value = argparse.Namespace(
            previous_tag="v0.19.0",
            baseline_ref=None,
            commit_range=None,
            dry_run=False,
            allow_orphan_footer=False,
        )
        resolve_validation_target.return_value = prep.ValidationTarget(
            baseline_ref="v0.19.0",
            baseline_label="v0.19.0",
            commit_range="v0.19.0..HEAD",
        )
        read_schema_version_at_ref.return_value = "0.13.0"
        read_current_schema_version.return_value = "0.14.0"
        read_schema_dir_contents_at_ref.return_value = {"style.json": "before"}
        read_schema_dir_contents.side_effect = [
            {"style.json": "checked-in"},
            {"style.json": "generated"},
        ]
        collect_schema_bump_markers.return_value = ([], [])

        result = prep.main([])

        self.assertEqual(result, 1)
        export_schemas.assert_called_once()


if __name__ == "__main__":
    unittest.main()
