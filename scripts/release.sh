#!/usr/bin/env bash
# scripts/release.sh — bump workspace version and prepare a release commit
#
# Usage:
#   ./scripts/release.sh patch|minor|major [--name "Release Name"] [--dry-run]
#   ./scripts/release.sh <X.Y.Z>           [--name "Release Name"] [--dry-run]
#
# Examples:
#   ./scripts/release.sh patch
#   ./scripts/release.sh minor --name "Andromeda"
#   ./scripts/release.sh major --dry-run
#   ./scripts/release.sh 1.0.0 --name "Andromeda"
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="$ROOT/Cargo.toml"
CHANGELOG="$ROOT/CHANGELOG.md"

# ── Parse arguments ───────────────────────────────────────────────────────────
BUMP_ARG=""
RELEASE_NAME=""
DRY_RUN=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --name)
      shift
      RELEASE_NAME="${1:-}"
      shift
      ;;
    --dry-run)
      DRY_RUN=true
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
  echo "Usage: $0 patch|minor|major [--name \"Release Name\"] [--dry-run]" >&2
  echo "       $0 <X.Y.Z>           [--name \"Release Name\"] [--dry-run]" >&2
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

# ── Print summary ─────────────────────────────────────────────────────────────
if [[ "$IS_MAJOR" == "true" ]]; then
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  MAJOR VERSION BUMP: $CURRENT_VERSION → $NEW_VERSION"
  [[ -n "$RELEASE_NAME" ]] && echo "  Name: $RELEASE_NAME"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
else
  echo "Release summary:"
  echo "  $CURRENT_VERSION → $NEW_VERSION  ($(date +%Y-%m-%d))"
  [[ -n "$RELEASE_NAME" ]] && echo "  Name: $RELEASE_NAME"
fi
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
  echo "Changelog preview (git cliff --unreleased):"
  git cliff --unreleased 2>/dev/null || echo "(git-cliff not available)"
  echo ""
  echo "(dry run — no files modified)"
  exit 0
fi

if [[ "$IS_MAJOR" == "true" ]]; then
  read -r -p "Type CONFIRM to proceed: " REPLY
  if [[ "$REPLY" != "CONFIRM" ]]; then
    echo "Aborted." >&2
    exit 1
  fi
  echo ""
fi

# ── Bump Cargo.toml ───────────────────────────────────────────────────────────
sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"

# ── Refresh Cargo.lock ────────────────────────────────────────────────────────
echo "Running cargo check to refresh Cargo.lock..."
cargo check --quiet --manifest-path "$CARGO_TOML" 2>&1

# ── Commit version bump ───────────────────────────────────────────────────────
git -C "$ROOT" add "$CARGO_TOML" "$ROOT/Cargo.lock"
git -C "$ROOT" commit -m "chore(release): bump workspace version to $NEW_VERSION"

# ── Tag the release ───────────────────────────────────────────────────────────
git -C "$ROOT" tag "v$NEW_VERSION"

# ── Regenerate CHANGELOG.md via git-cliff ─────────────────────────────────────
echo "Generating CHANGELOG.md with git-cliff..."
git cliff -o "$CHANGELOG" 2>/dev/null
git -C "$ROOT" add "$CHANGELOG"
git -C "$ROOT" commit --amend --no-edit

echo ""
echo "Done. Push when ready:"
echo "  git push origin main --tags"
