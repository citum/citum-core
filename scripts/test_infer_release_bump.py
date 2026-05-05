#!/usr/bin/env python3
"""Regression tests for release bump inference."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).resolve().parent / "infer-release-bump.py"
SPEC = importlib.util.spec_from_file_location("infer_release_bump", SCRIPT_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"Could not load release inference module from {SCRIPT_PATH}")
infer = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = infer
SPEC.loader.exec_module(infer)


class ConventionalLevelTests(unittest.TestCase):
    """Verify conventional commits map to release levels."""

    def test_fix_commit_infers_patch(self) -> None:
        level = infer.conventional_level(["fix(engine): keep delimiters"], "", "0.41.0")

        self.assertEqual(level, "patch")

    def test_perf_commit_infers_patch(self) -> None:
        level = infer.conventional_level(["perf(engine): cache names"], "", "0.41.0")

        self.assertEqual(level, "patch")

    def test_feat_commit_infers_minor(self) -> None:
        level = infer.conventional_level(["feat(schema): add registry urls"], "", "0.41.0")

        self.assertEqual(level, "minor")

    def test_breaking_pre_one_zero_caps_to_minor(self) -> None:
        level = infer.conventional_level(["fix!: replace API"], "", "0.41.0")

        self.assertEqual(level, "minor")

    def test_breaking_after_one_zero_stays_major(self) -> None:
        level = infer.conventional_level(["fix!: replace API"], "", "1.2.3")

        self.assertEqual(level, "major")

    def test_breaking_change_footer_infers_major(self) -> None:
        level = infer.conventional_level(
            ["fix(engine): replace API"],
            "BREAKING CHANGE: remove legacy resolver",
            "1.2.3",
        )

        self.assertEqual(level, "major")

    def test_chore_commit_has_no_release_impact(self) -> None:
        level = infer.conventional_level(["chore(ci): simplify hooks"], "", "0.41.0")

        self.assertEqual(level, "none")


class ReleaseImpactTests(unittest.TestCase):
    """Verify changed paths combine with commit levels correctly."""

    def test_schema_change_with_feat_releases_minor(self) -> None:
        impact = infer.infer_release_impact(
            ["crates/citum-schema-style/src/registry.rs", "docs/schemas/registry.json"],
            ["feat(schema): add URL-backed entries"],
            "",
            "0.41.0",
        )

        self.assertTrue(impact.should_release)
        self.assertTrue(impact.schema_changed)
        self.assertTrue(impact.code_changed)
        self.assertEqual(impact.level, "minor")

    def test_engine_fix_releases_patch(self) -> None:
        impact = infer.infer_release_impact(
            ["crates/citum-engine/src/render.rs"],
            ["fix(engine): render suffix spacing"],
            "",
            "0.41.0",
        )

        self.assertTrue(impact.should_release)
        self.assertTrue(impact.code_changed)
        self.assertFalse(impact.schema_changed)
        self.assertEqual(impact.level, "patch")

    def test_docs_only_feat_does_not_release(self) -> None:
        impact = infer.infer_release_impact(
            ["docs/specs/RESOLVER.md"],
            ["feat(schema): specify resolver"],
            "",
            "0.41.0",
        )

        self.assertFalse(impact.should_release)
        self.assertEqual(impact.level, "none")

    def test_schema_path_without_release_commit_does_not_release(self) -> None:
        impact = infer.infer_release_impact(
            ["docs/schemas/style.json"],
            ["chore(schema): regenerate output"],
            "",
            "0.41.0",
        )

        self.assertTrue(impact.schema_changed)
        self.assertFalse(impact.should_release)
        self.assertEqual(impact.level, "none")

    def test_github_output_uses_action_field_names(self) -> None:
        impact = infer.ReleaseImpact(
            level="patch",
            should_release=True,
            code_changed=True,
            schema_changed=False,
        )
        with tempfile.TemporaryDirectory() as tempdir:
            output = Path(tempdir) / "github-output"
            infer.write_github_output(output, impact)

            self.assertEqual(
                output.read_text(encoding="utf-8"),
                "level=patch\n"
                "should-release=true\n"
                "code-changed=true\n"
                "schema-changed=false\n",
            )


if __name__ == "__main__":
    unittest.main()
