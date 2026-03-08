#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
BEAN_WRAPPER="$ROOT_DIR/.claude/skills/beans/bin/citum-bean"

cd "$ROOT_DIR"

errors=0

say() {
  printf '%s\n' "$*"
}

fail() {
  say "ERROR: $*"
  errors=$((errors + 1))
}

check_broken_links() {
  say "[check] markdown relative links"
  local tmp
  tmp=$(mktemp)

  while IFS= read -r file; do
    local dir
    dir=$(dirname "$file")

    { grep -oE '\[[^]]+\]\(([^)]+)\)' "$file" || true; } \
      | sed -E 's/^[^\(]*\(([^)]+)\)$/\1/' \
      | while IFS= read -r raw_target; do
          local target
          target=${raw_target%%#*}
          target=${target%/}

          [ -z "$target" ] && continue
          case "$target" in
            http://*|https://*|mailto:*|tel:*|data:*) continue ;;
          esac

          local resolved
          if [[ "$target" = /* ]]; then
            resolved=".${target}"
          else
            resolved="${dir}/${target}"
          fi

          if [ ! -e "$resolved" ]; then
            printf '%s -> %s\n' "$file" "$raw_target" >> "$tmp"
          fi
        done
  done < <({
    find docs -type f -name '*.md' | sort
    for file in README.md CLAUDE.md; do
      [ -f "$file" ] && printf '%s\n' "$file"
    done
  })

  if [ -s "$tmp" ]; then
    fail "broken markdown links found"
    sort -u "$tmp"
  else
    say "ok: no broken markdown links"
  fi

  rm -f "$tmp"
}

check_bean_hygiene() {
  say "[check] bean hygiene"
  if "$BEAN_WRAPPER" hygiene; then
    say "ok: bean hygiene passed"
  else
    fail "bean hygiene failed"
  fi
}

check_broken_links
check_bean_hygiene

if [ "$errors" -ne 0 ]; then
  say "hygiene check failed with $errors error(s)"
  exit 1
fi

say "hygiene check passed"
