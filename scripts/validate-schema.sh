#!/usr/bin/env bash
# Backward-compatible wrapper for production style validation.
#
# Usage: ./scripts/validate-schema.sh [styles/*.yaml ...]

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

exec "$ROOT_DIR/scripts/validate-production-styles.sh" "$@"
