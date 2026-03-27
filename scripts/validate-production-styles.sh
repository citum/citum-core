#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"
BIN_PATH="$TARGET_DIR/debug/citum"

info() {
  printf '[INFO] %s\n' "$*"
}

error() {
  printf '[ERROR] %s\n' "$*" >&2
}

is_production_style() {
  local path="$1"
  [[ "$path" =~ ^styles/[^/]+\.yaml$ ]]
}

repo_rel() {
  local path="$1"
  path="${path#"$ROOT_DIR"/}"
  printf '%s\n' "$path"
}

collect_styles() {
  local -a requested=()
  local path rel

  if (( $# == 0 )); then
    while IFS= read -r path; do
      requested+=("$(repo_rel "$path")")
    done < <(find "$ROOT_DIR/styles" -maxdepth 1 -type f -name '*.yaml' | LC_ALL=C sort)
  else
    for path in "$@"; do
      rel="$(repo_rel "$path")"
      if ! is_production_style "$rel"; then
        continue
      fi
      if [[ ! -f "$ROOT_DIR/$rel" ]]; then
        error "Style file not found: $rel"
        exit 1
      fi
      requested+=("$rel")
    done
  fi

  if (( ${#requested[@]} == 0 )); then
    return 0
  fi

  printf '%s\n' "${requested[@]}" | LC_ALL=C sort -u
}

main() {
  local -a styles=()
  local style
  local failed=0
  local output
  local lint_status=0

  while IFS= read -r style; do
    [[ -n "$style" ]] && styles+=("$style")
  done < <(collect_styles "$@")

  if (( ${#styles[@]} == 0 )); then
    info "No production styles to validate."
    exit 0
  fi

  info "Building workspace citum binary"
  cargo build --quiet --bin citum --manifest-path "$ROOT_DIR/Cargo.toml"

  if [[ ! -x "$BIN_PATH" ]]; then
    error "Expected workspace binary at $BIN_PATH"
    exit 1
  fi

  info "Validating ${#styles[@]} production style(s)"
  for style in "${styles[@]}"; do
    if ! output="$("$BIN_PATH" check -s "$style" 2>&1)"; then
      failed=1
    fi
    printf '%s\n' "$output"
  done

  info "Linting style structure"
  if ! node "$ROOT_DIR/scripts/style-structure-lint.js" "${styles[@]}"; then
    lint_status=1
  fi

  if (( failed != 0 || lint_status != 0 )); then
    error "Production style validation failed."
    exit 1
  fi

  info "Production style validation passed."
}

main "$@"
