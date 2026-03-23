#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
git -C "$ROOT_DIR" config core.hooksPath .githooks
echo "Git hooks configured: core.hooksPath = .githooks"
echo "Hooks active: $(ls "$ROOT_DIR/.githooks/")"
