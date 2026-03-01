#!/usr/bin/env bash
# scripts/release.sh — bump workspace version and prepare a release commit
#
# Usage:
#   ./scripts/release.sh patch|minor|major [--name "Release Name"]
#   ./scripts/release.sh <X.Y.Z>           [--name "Release Name"]
#
# Examples:
#   ./scripts/release.sh patch
#   ./scripts/release.sh minor --name "Andromeda"
#   ./scripts/release.sh major
#   ./scripts/release.sh 1.0.0 --name "Andromeda"
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="$ROOT/Cargo.toml"
CHANGELOG="$ROOT/CHANGELOG.md"

# ── Parse arguments ───────────────────────────────────────────────────────────
BUMP_ARG=""
RELEASE_NAME=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --name)
      shift
      RELEASE_NAME="${1:-}"
      shift
      ;;
    -*)
      echo "Unknown flag: $1" >&2
      exit 1
      ;;
    *)
      if [[ -n "$BUMP_ARG" ]]; then
        echo "Error: unexpected argument '$1'" >&2
        exit 1
      fi
      BUMP_ARG="$1"
      shift
      ;;
  esac
done

if [[ -z "$BUMP_ARG" ]]; then
  echo "Usage: $0 patch|minor|major [--name \"Release Name\"]" >&2
  echo "       $0 <X.Y.Z>           [--name \"Release Name\"]" >&2
  exit 1
fi

# ── Read current version ──────────────────────────────────────────────────────
CURRENT_VERSION=$(grep -m1 '^version = ' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/')
CUR_MAJOR=$(echo "$CURRENT_VERSION" | cut -d. -f1)
CUR_MINOR=$(echo "$CURRENT_VERSION" | cut -d. -f2)
CUR_PATCH=$(echo "$CURRENT_VERSION" | cut -d. -f3)

# ── Resolve new version ───────────────────────────────────────────────────────
case "$BUMP_ARG" in
  patch)
    NEW_VERSION="$CUR_MAJOR.$CUR_MINOR.$((CUR_PATCH + 1))"
    IS_MAJOR=false
    ;;
  minor)
    NEW_VERSION="$CUR_MAJOR.$((CUR_MINOR + 1)).0"
    IS_MAJOR=false
    ;;
  major)
    NEW_VERSION="$((CUR_MAJOR + 1)).0.0"
    IS_MAJOR=true
    ;;
  *)
    if ! [[ "$BUMP_ARG" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
      echo "Error: '$BUMP_ARG' is not patch/minor/major or a valid semver (X.Y.Z)" >&2
      exit 1
    fi
    NEW_VERSION="$BUMP_ARG"
    NEW_MAJOR=$(echo "$NEW_VERSION" | cut -d. -f1)
    IS_MAJOR=$([[ "$NEW_MAJOR" -gt "$CUR_MAJOR" ]] && echo true || echo false)
    ;;
esac

# ── Check working tree is clean ───────────────────────────────────────────────
if ! git -C "$ROOT" diff --quiet || ! git -C "$ROOT" diff --cached --quiet; then
  echo "Error: working tree has uncommitted changes — commit or stash first" >&2
  exit 1
fi

# ── Extract [Unreleased] content ──────────────────────────────────────────────
UNRELEASED=$(awk '/^## \[Unreleased\]/{found=1; next} found && /^## \[/{exit} found{print}' "$CHANGELOG" | sed '/^[[:space:]]*$/d')

# ── Major confirmation ────────────────────────────────────────────────────────
if [[ "$IS_MAJOR" == "true" ]]; then
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  MAJOR VERSION BUMP: $CURRENT_VERSION → $NEW_VERSION"
  [[ -n "$RELEASE_NAME" ]] && echo "  Name: $RELEASE_NAME"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  if [[ -n "$UNRELEASED" ]]; then
    echo "Unreleased changes included in this release:"
    echo ""
    echo "$UNRELEASED"
    echo ""
  else
    echo "(No unreleased changes documented in CHANGELOG.md)"
    echo ""
  fi
  read -r -p "Type CONFIRM to proceed: " REPLY
  if [[ "$REPLY" != "CONFIRM" ]]; then
    echo "Aborted." >&2
    exit 1
  fi
  echo ""
else
  # Dry-run summary for patch/minor
  echo "Release summary:"
  echo "  $CURRENT_VERSION → $NEW_VERSION  ($(date +%Y-%m-%d))"
  [[ -n "$RELEASE_NAME" ]] && echo "  Name: $RELEASE_NAME"
  echo ""
fi

# ── Bump Cargo.toml ───────────────────────────────────────────────────────────
sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"

# ── Update CHANGELOG.md ───────────────────────────────────────────────────────
TODAY=$(date +%Y-%m-%d)
HEADING="## [$NEW_VERSION] - $TODAY"
[[ -n "$RELEASE_NAME" ]] && HEADING="$HEADING — $RELEASE_NAME"

if ! grep -q '## \[Unreleased\]' "$CHANGELOG"; then
  echo "Error: CHANGELOG.md missing '## [Unreleased]' section" >&2
  exit 1
fi

# Promote [Unreleased] → versioned heading
sed -i '' "s|## \[Unreleased\]|$HEADING|" "$CHANGELOG"

# Prepend fresh [Unreleased] section
sed -i '' "s|$HEADING|## [Unreleased]\n\n$HEADING|" "$CHANGELOG"

# ── Refresh Cargo.lock ────────────────────────────────────────────────────────
echo "Running cargo check to refresh Cargo.lock..."
cargo check --quiet --manifest-path "$CARGO_TOML" 2>&1

# ── Commit ────────────────────────────────────────────────────────────────────
git -C "$ROOT" add -A
git -C "$ROOT" commit -m "chore(release): bump workspace version to $NEW_VERSION"

echo ""
echo "Done. Push when ready:"
echo "  git push origin main"
