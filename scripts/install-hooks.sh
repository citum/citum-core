#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
git -C "$ROOT_DIR" config core.hooksPath .githooks
echo "Git hooks configured: core.hooksPath = .githooks"
echo ""
echo "Installed hooks:"
echo "  pre-commit: Validates production styles if styles/*.yaml changed"
echo "  commit-msg: Enforces conventional commits format"
echo "  pre-push:   Rejects Co-Authored-By footers; validates commit scopes against allowlist; runs cargo fmt/clippy/nextest if .rs or Cargo.* files changed"
echo ""
echo "Active hooks: $(ls "$ROOT_DIR/.githooks/")"
