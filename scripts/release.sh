#!/usr/bin/env bash

set -euo pipefail

echo "scripts/release.sh is deprecated in this repository." >&2
echo "Code releases are managed by release-plz." >&2
echo "Use ./scripts/bump.sh schema <patch|minor|major> for schema-only bumps." >&2
exit 1
