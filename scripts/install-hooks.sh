#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
git -C "$ROOT_DIR" config core.hooksPath .githooks
echo "Git hooks configured: core.hooksPath = .githooks"
echo ""
echo "Installed hooks:"
echo "  pre-commit: Schema re-gen + bean hygiene"
echo "  commit-msg: Conventional commit format + Schema-Bump footer"
echo "  pre-push:   Co-Authored-By check; scope allowlist; schema contract; cargo fmt/clippy/nextest"
echo ""
echo "Active hooks: $(ls "$ROOT_DIR/.githooks/")"
