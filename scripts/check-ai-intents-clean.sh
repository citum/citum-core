#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT_DIR"

tracked=$(git ls-files '.ai-intents' '.ai-intents/**')
untracked=$(git ls-files --others --exclude-standard '.ai-intents' '.ai-intents/**')

if [[ -n "$tracked$untracked" ]]; then
  cat >&2 <<'EOF'
ERROR: .ai-intents/ contains publishable files.

.ai-intents/ is temporary local drafting provenance for jj-assisted work.
Delete those files before exporting, pushing, or opening a PR unless the user
explicitly requested durable prompt provenance.
EOF
  {
    printf '%s\n' "$tracked"
    printf '%s\n' "$untracked"
  } | sed '/^$/d' >&2
  exit 1
fi

echo "ok: no .ai-intents/ files are publishable"
