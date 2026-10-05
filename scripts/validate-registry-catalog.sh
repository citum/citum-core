#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REGISTRY="$ROOT_DIR/crates/citum-schema-style/embedded/registry/default.yaml"
BIN_PATH="${CITUM_BIN:-$ROOT_DIR/target/debug/citum}"
CATALOG_DIR="${1:-}"
TEMP_DIR=""
EXPECTED_CATALOG_COUNT=122

cleanup() {
  if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
    rm -rf -- "$TEMP_DIR"
  fi
}
trap cleanup EXIT

if [[ -z "${CITUM_BIN:-}" ]]; then
  cargo build --quiet --manifest-path "$ROOT_DIR/Cargo.toml" --bin citum
elif [[ ! -x "$BIN_PATH" ]]; then
  printf 'CITUM_BIN is not executable: %s\n' "$BIN_PATH" >&2
  exit 1
fi

CATALOG_REF="$(sed -n \
  's#^  url: https://raw.githubusercontent.com/citum/citum-styles/\([^/]*\)/styles/.*#\1#; T; p; q' \
  "$REGISTRY")"
if [[ -z "$CATALOG_REF" ]]; then
  printf 'No citum-styles registry source found.\n' >&2
  exit 1
fi

if [[ -z "$CATALOG_DIR" ]]; then
  TEMP_DIR="$(mktemp -d)"
  ARCHIVE="$TEMP_DIR/catalog.tar.gz"
  curl --fail --silent --show-error --location \
    "https://codeload.github.com/citum/citum-styles/tar.gz/$CATALOG_REF" \
    --output "$ARCHIVE"
  tar -xzf "$ARCHIVE" -C "$TEMP_DIR"
  CATALOG_DIR="$(find "$TEMP_DIR" -mindepth 1 -maxdepth 1 -type d | head -1)"
fi

failures=()
builtins=0
validated=0
while IFS= read -r builtin; do
  [[ -n "$builtin" ]] || continue
  if ! "$BIN_PATH" style validate "$builtin" \
    --format json --include-resolved >/dev/null 2>&1; then
    failures+=("invalid builtin style: $builtin")
    continue
  fi
  builtins=$((builtins + 1))
done < <(sed -n 's/^  builtin: //p' "$REGISTRY")

while IFS= read -r relative; do
  [[ -n "$relative" ]] || continue
  style_path="$CATALOG_DIR/styles/$relative"
  if [[ ! -f "$style_path" ]]; then
    failures+=("missing catalog style: $relative")
    continue
  fi
  if ! "$BIN_PATH" style validate "$style_path" \
    --format json --include-resolved >/dev/null 2>&1; then
    failures+=("invalid catalog style: $relative")
    continue
  fi
  validated=$((validated + 1))
done < <(sed -n \
  's#^  url: https://raw.githubusercontent.com/citum/citum-styles/[^/]*/styles/\(.*\)$#\1#p' \
  "$REGISTRY")

if (( validated != EXPECTED_CATALOG_COUNT )); then
  failures+=("expected $EXPECTED_CATALOG_COUNT valid catalog styles, found $validated")
fi

if (( ${#failures[@]} > 0 )); then
  printf '%s\n' "${failures[@]}" >&2
  exit 1
fi

printf 'Validated %d builtins and %d pinned citum-styles registry entries at %s.\n' \
  "$builtins" "$validated" "$CATALOG_REF"
