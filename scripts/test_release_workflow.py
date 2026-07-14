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
SCHEMA_LIB = REPO_ROOT / "crates/citum-schema-style/src/version.rs"
INSTALL_SCRIPT = REPO_ROOT / "scripts/install.sh"
PUBLISH_CRATES_SCRIPT = REPO_ROOT / "scripts/publish-crates.sh"
BUILD_JSR_SCRIPT = REPO_ROOT / "scripts/build-jsr-package.sh"
JSR_README_SOURCE = REPO_ROOT / "crates/citum-bindings/README-JSR.md"


class ReleaseWorkflowTests(unittest.TestCase):
    """Ensure release workflow semantics stay aligned with policy."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
        cls.release_config = RELEASE_CONFIG_PATH.read_text(encoding="utf-8")
        cls.install_script = INSTALL_SCRIPT.read_text(encoding="utf-8")
        cls.publish_crates_script = PUBLISH_CRATES_SCRIPT.read_text(encoding="utf-8")
        cls.build_jsr_script = BUILD_JSR_SCRIPT.read_text(encoding="utf-8")
        cls.jsr_readme_source = JSR_README_SOURCE.read_text(encoding="utf-8")

    def test_release_branch_is_always_release_next(self) -> None:
        self.assertIn('echo "branch=release/next"', self.workflow)
        self.assertNotIn('echo "branch=main"', self.workflow)

    def test_release_pr_is_always_enabled(self) -> None:
        self.assertIn('echo "create_pr=true"', self.workflow)
        self.assertNotIn('echo "create_pr=false"', self.workflow)

    def test_cargo_release_does_not_use_metadata_flag_for_commit_message(self) -> None:
        bump_workspace_block = re.search(
            r"- name: Bump workspace version.*?cargo release .*?\n",
            self.workflow,
            flags=re.DOTALL,
        )
        self.assertIsNotNone(bump_workspace_block)
        assert bump_workspace_block is not None
        self.assertNotIn(" -m ", bump_workspace_block.group(0))

    def test_release_config_does_not_hardcode_changelog_output(self) -> None:
        self.assertNotIn('"git-cliff", "-o", "CHANGELOG.md"', self.release_config)

    def test_publish_crates_dry_run_accepts_current_dependency_gap_wording(self) -> None:
        """Cargo may report unpublished internal deps as version selection gaps."""
        self.assertIn(
            "failed to select a version for the requirement \\`$crate =",
            self.publish_crates_script,
        )
        self.assertIn(
            "candidate versions found which didn't match",
            self.publish_crates_script,
        )
        self.assertIn(
            "no matching package named \\`$crate\\` found",
            self.publish_crates_script,
        )

    def test_jsr_package_license_uses_jsr_supported_identifier(self) -> None:
        self.assertIn('"license": "MIT"', self.build_jsr_script)
        self.assertNotIn('"license": "(MIT OR Apache-2.0)"', self.build_jsr_script)
        self.assertNotIn('"license": "MIT OR Apache-2.0"', self.build_jsr_script)

    def test_jsr_package_uses_package_specific_readme(self) -> None:
        self.assertIn("crates/citum-bindings/README-JSR.md", self.build_jsr_script)
        self.assertNotIn('cp "$REPO_ROOT/README.md"', self.build_jsr_script)
        self.assertIn("# @citum/engine", self.jsr_readme_source)
        self.assertIn("Package metadata is `MIT` for JSR compatibility", self.jsr_readme_source)

    def test_publish_jsr_tag_job_uses_oidc_and_publishes(self) -> None:
        publish_jsr = re.search(
            r"\n  publish-jsr:\n(?P<block>.*?)(?=\n  [a-zA-Z0-9_-]+:|\Z)",
            self.workflow,
            flags=re.DOTALL,
        )
        self.assertIsNotNone(publish_jsr)
        assert publish_jsr is not None
        block = publish_jsr.group("block")

        self.assertIn("startsWith(github.ref, 'refs/tags/v')", block)
        self.assertIn("inputs.command == 'publish-jsr'", block)
        self.assertNotIn("needs: build", block)
        self.assertIn("id-token: write", block)
        self.assertIn("run: ./scripts/build-jsr-package.sh", block)
        self.assertIn("working-directory: target/jsr/citum", block)
        self.assertIn("run: npx --yes jsr publish --dry-run", block)
        self.assertRegex(
            block,
            r"- name: Publish to JSR[\s\S]*?run: npx --yes jsr publish",
            "publish-jsr must include a real publish step after dry-run",
        )

    def test_release_workflow_has_manual_publish_recovery_commands(self) -> None:
        self.assertIn("- publish-jsr", self.workflow)
        self.assertIn("- publish-crates", self.workflow)

    def test_manual_crates_publish_recovery_runs_the_build_gate(self) -> None:
        """Manual crates.io recovery must retain the release build gate."""
        build = re.search(
            r"\n  build:\n(?P<block>.*?)(?=\n  [a-zA-Z0-9_-]+:|\Z)",
            self.workflow,
            flags=re.DOTALL,
        )
        publish = re.search(
            r"\n  publish-crates:\n(?P<block>.*?)(?=\n  [a-zA-Z0-9_-]+:|\Z)",
            self.workflow,
            flags=re.DOTALL,
        )
        self.assertIsNotNone(build)
        self.assertIsNotNone(publish)
        assert build is not None
        assert publish is not None
        self.assertIn("inputs.command == 'publish-crates'", build.group("block"))
        self.assertIn("inputs.command == 'publish-crates'", publish.group("block"))
        self.assertIn("needs: build", publish.group("block"))
        self.assertIn("release_ref", self.workflow)
        self.assertIn("publish-crates requires release_ref", build.group("block"))
        self.assertIn("ref: ${{ inputs.release_ref || github.ref }}", build.group("block"))
        self.assertIn("ref: ${{ inputs.release_ref || github.ref }}", publish.group("block"))

    def test_release_binary_matrix_drops_intel_macos_but_keeps_windows(self) -> None:
        self.assertNotIn("target: x86_64-apple-darwin", self.workflow)
        self.assertIn("target: aarch64-apple-darwin", self.workflow)
        self.assertIn("target: x86_64-pc-windows-msvc", self.workflow)

    def test_release_binary_matrix_keeps_only_x86_64_gnu_migrate_target(self) -> None:
        """rusty_v8 (citum-migrate's V8 dep) has no musl prebuilt but does
        have an x86_64 gnu one — see issue #1054. The matrix must retain that
        target without requiring the unsupported ARM GNU V8 link."""
        self.assertIn("target: x86_64-unknown-linux-gnu", self.workflow)
        self.assertNotIn("target: aarch64-unknown-linux-gnu", self.workflow)
        # musl targets must still be present — they remain the default for
        # citum/citum-server.
        self.assertIn("target: x86_64-unknown-linux-musl", self.workflow)
        self.assertIn("target: aarch64-unknown-linux-musl", self.workflow)

    def test_installer_fetches_migrate_from_x86_64_gnu_fallback_on_musl(self) -> None:
        """install.sh must not silently drop citum-migrate on Linux; it
        should fetch the binary from the gnu tarball instead (issue #1054)."""
        self.assertIn("migrate_fallback_target", self.install_script)
        self.assertIn("x86_64-unknown-linux-gnu", self.install_script)
        self.assertNotIn("aarch64-unknown-linux-gnu", self.install_script)
        self.assertIn("fetch_tarball", self.install_script)
        self.assertIn('if [ -n "$MIGRATE_TARGET" ]; then', self.install_script)
        # The graceful degrade path (gnu fetch itself unavailable) must
        # remain, so a stale/offline mirror doesn't hard-fail the install.
        self.assertIn("cargo install citum-migrate --locked", self.install_script)

    def test_installer_does_not_map_intel_macos_to_missing_tarball(self) -> None:
        self.assertNotIn('echo "x86_64-apple-darwin"', self.install_script)
        self.assertIn("prebuilt Intel macOS binaries are no longer shipped", self.install_script)
        self.assertIn("cargo install citum --locked", self.install_script)
        self.assertIn("cargo install citum-server --locked", self.install_script)
        self.assertIn('arm64)               echo "aarch64-apple-darwin"', self.install_script)
        self.assertIn("sysctl.proc_translated", self.install_script)

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
