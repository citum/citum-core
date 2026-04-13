#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-crates}"

COMMON_GLOBS=(
  --glob '*.rs'
  --glob '!**/tests/**'
  --glob '!**/benches/**'
  --glob '!**/examples/**'
  --glob '!**/target/**'
)

echo "== Existing string aliases and wrappers =="
rg -n "${COMMON_GLOBS[@]}" \
  '^\s*pub\s+(type\s+[A-Z][A-Za-z0-9_]*\s*=\s*String;|struct\s+[A-Z][A-Za-z0-9_]*\s*\(\s*(pub\s+)?String\s*\);)' \
  "$ROOT"

echo
echo "== Public struct fields with semantic primitive names =="
rg -n "${COMMON_GLOBS[@]}" \
  '\bpub\s+(id|slug|locale|language|label|name|title|path|url|uri)\s*:\s*(Option<)?String' \
  "$ROOT"

echo
echo "== Public functions with semantic primitive params =="
rg -n "${COMMON_GLOBS[@]}" \
  'pub\s+fn\s+\w+[[:space:]]*(<[^>]+>)?[[:space:]]*\(([^)]*(id|slug|locale|language|label|name|path|url|uri)\s*:\s*&?str[^)]*|[^)]*(id|slug|locale|language|label|name|path|url|uri)\s*:\s*String[^)]*)\)' \
  "$ROOT"

echo
echo "== Primitive-heavy public structs (2+ primitive fields) =="
find "$ROOT" -path '*/tests/*' -prune -o -path '*/benches/*' -prune -o -name '*.rs' -print \
| xargs perl -0ne '
  while(/pub\s+struct\s+(\w+)[^{]*\{([^}]*)\}/sg){
    my $name=$1;
    my $body=$2;
    my $count=()=($body =~ /:\s*(?:String|bool|char|u(?:8|16|32|64|size)|i(?:8|16|32|64|size)|f(?:32|64)|usize|isize)\b/g);
    print "$ARGV:$name:$count\n" if $count >= 2;
  }
' \
| sort -t: -k3,3nr
