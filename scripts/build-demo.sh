#!/usr/bin/env bash
# Regenerate docs/demo.html from docs/demo.djot.
# Usage: ./scripts/build-demo.sh [style]
#   style defaults to styles/embedded/chicago-author-date-18th.yaml
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/.."

STYLE="${1:-styles/embedded/chicago-author-date-18th.yaml}"
DOC="docs/demo.djot"
REFS="docs/demo-refs.yaml"
OUT="docs/demo.html"

cd "$ROOT"

TMP=$(mktemp)
trap "rm -f $TMP" EXIT

# Render; strip the in-document preamble (notice + features paragraph + two <hr>s)
# that the engine emits — the template already has those in the page wrapper.
# Also: (1) drop the document-title <h1> from the article section (the page
#     template shows it in the <header>); (2) add class="content" so the
#     .content p+p paragraph-indentation rule in citum-interactive.css applies.
cargo run --bin citum -- render doc "$DOC" -b "$REFS" -s "$STYLE" -f html 2>/dev/null \
  | sed -n '/<section/,$p' \
  | sed 's|id="The-Infrastructure-of-Scholarly-Memory"|id="The-Infrastructure-of-Scholarly-Memory" class="content"|' \
  | sed '/^<h1>The Infrastructure of Scholarly Memory<\/h1>$/d' \
  > "$TMP"

# Inject rendered content between the CONTENT_START / CONTENT_END markers.
python3 - "$OUT" "$TMP" <<'PYEOF'
import sys, re

out_path, content_path = sys.argv[1], sys.argv[2]
with open(content_path) as f:
    new_content = f.read().rstrip()
with open(out_path) as f:
    html = f.read()

pattern = r'<!-- CONTENT_START -->.*?<!-- CONTENT_END -->'
if not re.search(pattern, html, re.DOTALL):
    print("ERROR: CONTENT_START/END markers not found in demo.html", file=sys.stderr)
    sys.exit(1)
replacement = '<!-- CONTENT_START -->\n' + new_content + '\n      <!-- CONTENT_END -->'
result = re.sub(pattern, replacement, html, flags=re.DOTALL)

with open(out_path, 'w') as f:
    f.write(result)
PYEOF

STYLE_NAME=$(basename "$STYLE" .yaml)
echo "✓ $OUT regenerated (style: $STYLE_NAME)"
