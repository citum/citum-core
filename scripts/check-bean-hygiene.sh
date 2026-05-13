#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
BEAN_WRAPPER="${BEAN_WRAPPER:-$ROOT_DIR/.claude/skills/beans/bin/citum-bean}"

cd "$ROOT_DIR"

if IGNORE_SOFT_STALE="${IGNORE_SOFT_STALE:-1}" "$BEAN_WRAPPER" hygiene; then
  echo "ok: bean hygiene passed"
else
  echo "ERROR: bean hygiene failed"
  exit 1
fi
