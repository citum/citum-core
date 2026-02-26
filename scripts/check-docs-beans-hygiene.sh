#!/usr/bin/env bash
set -euo pipefail

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
  done < <(
    find docs -type f -name '*.md' | sort
    printf '%s\n' README.md CLAUDE.md
  )

  if [ -s "$tmp" ]; then
    fail "broken markdown links found"
    sort -u "$tmp"
  else
    say "ok: no broken markdown links"
  fi

  rm -f "$tmp"
}

check_bean_statuses() {
  say "[check] bean status taxonomy"
  local invalid
  invalid=$(rg -n '^status: ' .beans/*.md | rg -v 'status: (todo|in-progress|completed|canceled)$' || true)
  if [ -n "$invalid" ]; then
    fail "invalid bean statuses found"
    printf '%s\n' "$invalid"
  else
    say "ok: all bean statuses valid"
  fi
}

check_duplicate_migrate_beans() {
  say "[check] duplicate in-progress migrate beans"
  local dupes
  dupes=$(for f in .beans/*.md; do
    status=$(sed -n 's/^status: //p' "$f" | head -n1)
    title=$(sed -n 's/^title: //p' "$f" | head -n1)
    if [ "$status" = "in-progress" ] && [[ "$title" == "'Migrate: "* ]]; then
      style=${title#\'Migrate: }
      style=${style%\'}
      printf '%s\t%s\n' "$style" "$(basename "$f")"
    fi
  done | sort | awk -F '\t' '{c[$1]++; ids[$1]=ids[$1]" "$2} END {for (k in c) if (c[k] > 1) print c[k]"\t"k"\t"ids[k]}' | sort -nr)

  if [ -n "$dupes" ]; then
    fail "duplicate in-progress migrate beans found"
    printf '%s\n' "$dupes"
  else
    say "ok: no duplicate in-progress migrate beans"
  fi
}

check_broken_links
check_bean_statuses
check_duplicate_migrate_beans

if [ "$errors" -ne 0 ]; then
  say "hygiene check failed with $errors error(s)"
  exit 1
fi

say "hygiene check passed"
