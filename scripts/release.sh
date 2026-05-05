#!/usr/bin/env bash

set -euo pipefail

echo "scripts/release.sh is deprecated in this repository." >&2
echo "Code and schema releases are managed by the release workflow." >&2
echo "Use conventional commits; do not run manual version bumps in feature PRs." >&2
exit 1
