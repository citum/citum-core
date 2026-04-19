#!/usr/bin/env python3
"""Regression tests for workspace version release validation helpers."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path
from unittest import mock

SCRIPT_PATH = Path(__file__).resolve().parent / "validate-version-release.py"
SPEC = importlib.util.spec_from_file_location("validate_version_release", SCRIPT_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"Could not load module from {SCRIPT_PATH}")
vvr = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = vvr
SPEC.loader.exec_module(vvr)


class BumpVersionTests(unittest.TestCase):
    """Verify semantic version arithmetic."""

    def test_patch_increments_patch(self) -> None:
        self.assertEqual(vvr.bump_version("0.20.1", "patch"), "0.20.2")

    def test_minor_increments_minor_resets_patch(self) -> None:
        self.assertEqual(vvr.bump_version("0.20.1", "minor"), "0.21.0")

    def test_major_pre_one_zero_increments_minor(self) -> None:
        # Pre-1.0: breaking changes fold into minor to stay in 0.x.y.
        self.assertEqual(vvr.bump_version("0.20.1", "major"), "0.21.0")

    def test_major_post_one_zero_increments_major(self) -> None:
        self.assertEqual(vvr.bump_version("1.2.3", "major"), "2.0.0")

    def test_unsupported_bump_type_raises(self) -> None:
        with self.assertRaises(vvr.ValidationError):
            vvr.bump_version("0.20.1", "ultraviolet")


class MarkerParsingTests(unittest.TestCase):
    """Verify Version-Bump footer collection from git log output."""

    def _run_collect(self, log_output: str) -> list[vvr.VersionBumpMarker]:
        with mock.patch.object(vvr, "run") as mock_run:
            mock_run.return_value = mock.Mock(stdout=log_output)
            return vvr.collect_version_bump_markers("abc..HEAD")

    def test_collects_single_footer(self) -> None:
        log = (
            f"abc1234\x1fsomething useful\x1f"
            f"something useful\n\nVersion-Bump: minor\n\x1e"
        )
        markers = self._run_collect(log)
        self.assertEqual(len(markers), 1)
        self.assertEqual(markers[0].bump, "minor")
        self.assertEqual(markers[0].commit, "abc1234"[:8])

    def test_collects_multiple_footers_across_commits(self) -> None:
        log = (
            f"aaa00001\x1ffirst commit\x1ffirst commit\n\nVersion-Bump: patch\n\x1e"
            f"bbb00002\x1fsecond commit\x1fsecond commit\n\nVersion-Bump: major\n\x1e"
        )
        markers = self._run_collect(log)
        self.assertEqual(len(markers), 2)
        bumps = {m.bump for m in markers}
        self.assertIn("patch", bumps)
        self.assertIn("major", bumps)

    def test_commit_with_no_footer_produces_no_marker(self) -> None:
        log = f"ccc00003\x1fjust a refactor\x1fjust a refactor\n\nNo footer here.\n\x1e"
        markers = self._run_collect(log)
        self.assertEqual(markers, [])

    def test_duplicate_footer_in_one_commit_raises(self) -> None:
        log = (
            f"ddd00004\x1fbad commit\x1f"
            f"bad commit\n\nVersion-Bump: patch\nVersion-Bump: minor\n\x1e"
        )
        with self.assertRaises(vvr.ValidationError):
            self._run_collect(log)

    def test_empty_range_returns_empty(self) -> None:
        markers = self._run_collect("")
        self.assertEqual(markers, [])


class MainValidationTests(unittest.TestCase):
    """End-to-end tests for main() with mocked git interactions."""

    def _run_main(
        self,
        *,
        changed_files: list[str],
        previous_version: str,
        current_version: str,
        markers: list[vvr.VersionBumpMarker],
        allow_orphan_footer: bool = False,
    ) -> int:
        argv = [
            "--baseline-ref", "origin/main",
            "--commit-range", "origin/main..HEAD",
        ]
        if allow_orphan_footer:
            argv.append("--allow-orphan-footer")

        with (
            mock.patch.object(vvr, "list_changed_files", return_value=changed_files),
            mock.patch.object(vvr, "read_workspace_version_at_ref", return_value=previous_version),
            mock.patch.object(vvr, "read_current_workspace_version", return_value=current_version),
            mock.patch.object(vvr, "collect_version_bump_markers", return_value=markers),
        ):
            return vvr.main(argv)

    def test_no_changes_no_markers_passes(self) -> None:
        rc = self._run_main(
            changed_files=[],
            previous_version="0.20.1",
            current_version="0.20.1",
            markers=[],
        )
        self.assertEqual(rc, 0)

    def test_patch_bump_passes(self) -> None:
        rc = self._run_main(
            changed_files=["crates/citum-engine/src/lib.rs"],
            previous_version="0.20.1",
            current_version="0.20.2",
            markers=[vvr.VersionBumpMarker("abc12345", "fix: thing", "patch")],
        )
        self.assertEqual(rc, 0)

    def test_minor_bump_passes(self) -> None:
        rc = self._run_main(
            changed_files=["crates/citum-engine/src/lib.rs"],
            previous_version="0.20.1",
            current_version="0.21.0",
            markers=[vvr.VersionBumpMarker("abc12345", "feat: new api", "minor")],
        )
        self.assertEqual(rc, 0)

    def test_highest_marker_wins_for_version_comparison(self) -> None:
        markers = [
            vvr.VersionBumpMarker("aaa", "fix: patch fix", "patch"),
            vvr.VersionBumpMarker("bbb", "feat!: breaking", "major"),
        ]
        # Pre-1.0: major folds to minor → expected version is 0.21.0
        rc = self._run_main(
            changed_files=["crates/citum-engine/src/lib.rs"],
            previous_version="0.20.1",
            current_version="0.21.0",
            markers=markers,
        )
        self.assertEqual(rc, 0)

    def test_version_changed_without_footer_fails(self) -> None:
        rc = self._run_main(
            changed_files=["crates/citum-engine/src/lib.rs"],
            previous_version="0.20.1",
            current_version="0.20.2",
            markers=[],
        )
        self.assertNotEqual(rc, 0)

    def test_footer_without_version_change_fails(self) -> None:
        rc = self._run_main(
            changed_files=["crates/citum-engine/src/lib.rs"],
            previous_version="0.20.1",
            current_version="0.20.1",
            markers=[vvr.VersionBumpMarker("abc12345", "feat: thing", "minor")],
        )
        self.assertNotEqual(rc, 0)

    def test_version_mismatch_fails(self) -> None:
        # Footer says patch (→ 0.20.2) but Cargo.toml was bumped to 0.21.0.
        rc = self._run_main(
            changed_files=["crates/citum-engine/src/lib.rs"],
            previous_version="0.20.1",
            current_version="0.21.0",
            markers=[vvr.VersionBumpMarker("abc12345", "fix: thing", "patch")],
        )
        self.assertNotEqual(rc, 0)

    def test_relevant_change_no_footer_no_version_fails(self) -> None:
        # Publishable crate changed but nothing else happened — should fail, not crash.
        rc = self._run_main(
            changed_files=["crates/citum-engine/src/lib.rs"],
            previous_version="0.20.1",
            current_version="0.20.1",
            markers=[],
        )
        self.assertNotEqual(rc, 0)

    def test_orphan_footer_allowed_when_flag_set(self) -> None:
        # No changes in range but one footer present — rescue scenario.
        rc = self._run_main(
            changed_files=[],
            previous_version="0.20.1",
            current_version="0.20.1",
            markers=[vvr.VersionBumpMarker("abc12345", "fix: rescue", "patch")],
            allow_orphan_footer=True,
        )
        self.assertEqual(rc, 0)

    def test_orphan_footer_rejected_without_flag(self) -> None:
        rc = self._run_main(
            changed_files=[],
            previous_version="0.20.1",
            current_version="0.20.1",
            markers=[vvr.VersionBumpMarker("abc12345", "fix: rescue", "patch")],
            allow_orphan_footer=False,
        )
        self.assertNotEqual(rc, 0)

    def test_non_publishable_crate_change_ignored(self) -> None:
        # citum-analyze is excluded; no bump required.
        rc = self._run_main(
            changed_files=["crates/citum-analyze/src/lib.rs"],
            previous_version="0.20.1",
            current_version="0.20.1",
            markers=[],
        )
        self.assertEqual(rc, 0)


if __name__ == "__main__":
    unittest.main()
