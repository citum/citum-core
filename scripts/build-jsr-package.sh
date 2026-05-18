#!/usr/bin/env bash
# Build the Citum WASM/TypeScript package for JSR.
#
# The Rust crate remains the source of truth. This script stages the generated
# wasm-bindgen output plus JSR package metadata under target/jsr/citum.

set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
CRATE_DIR="$REPO_ROOT/crates/citum-bindings"
PACKAGE_DIR="$REPO_ROOT/target/jsr/citum"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "error: wasm-pack is required. Install it with \`cargo install wasm-pack\`." >&2
  exit 1
fi

VERSION=$(sed -n '/^\[workspace\.package\]/,/^\[/{s/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p;}' "$REPO_ROOT/Cargo.toml" | head -1)
if [[ "$VERSION" == "" ]]; then
  echo "error: workspace package version not found in Cargo.toml" >&2
  exit 1
fi

rm -rf "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR"

wasm-pack build "$CRATE_DIR" \
  --target web \
  --out-dir "$PACKAGE_DIR" \
  --release \
  --features full-wasm

cp "$REPO_ROOT/README.md" "$PACKAGE_DIR/README.md"
cp "$REPO_ROOT/LICENSE" "$PACKAGE_DIR/LICENSE"
cp "$REPO_ROOT/LICENSE-APACHE" "$PACKAGE_DIR/LICENSE-APACHE"

cat > "$PACKAGE_DIR/jsr.json" <<JSON
{
  "name": "@citum/citum",
  "version": "$VERSION",
  "license": "MIT OR Apache-2.0",
  "exports": {
    ".": "./citum_bindings.js"
  },
  "publish": {
    "include": [
      "citum_bindings.js",
      "citum_bindings.d.ts",
      "citum_bindings_bg.wasm",
      "citum_bindings_bg.wasm.d.ts",
      "README.md",
      "LICENSE",
      "LICENSE-APACHE"
    ]
  }
}
JSON

echo "==> Built JSR package at $PACKAGE_DIR"
