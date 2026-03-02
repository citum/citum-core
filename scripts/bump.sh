#!/usr/bin/env bash
# Unified version bump script for Citum schema and engine tracks
#
# Usage: ./scripts/bump.sh [track] [bump-type] [--dry-run]
#        ./scripts/bump.sh [bump-type] [--dry-run]
#
# track: schema | engine | all (default: all)
# bump-type: patch | minor | major (required)
# --dry-run: preview changes without modifying files
#
# Examples:
#   ./scripts/bump.sh patch                      # Bump both tracks by patch
#   ./scripts/bump.sh schema minor               # Bump schema by minor
#   ./scripts/bump.sh patch --dry-run            # Preview both tracks patch bump
#   ./scripts/bump.sh engine major --dry-run     # Preview engine major bump

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; }

# Parse arguments in any order
TRACK="all"
BUMP_TYPE=""
DRY_RUN=false
RELEASE_NAME=""

args=("$@")
i=0
while [ $i -lt ${#args[@]} ]; do
    arg="${args[$i]}"
    case "$arg" in
        schema|engine|all)
            TRACK="$arg"
            ;;
        patch|minor|major)
            BUMP_TYPE="$arg"
            ;;
        --dry-run)
            DRY_RUN=true
            ;;
        --name)
            i=$((i + 1))
            RELEASE_NAME="${args[$i]}"
            ;;
        *)
            error "Unknown argument: $arg"
            error "Valid tracks: schema, engine, all"
            error "Valid bump types: patch, minor, major"
            error "Valid flags: --dry-run, --name <title>"
            exit 1
            ;;
    esac
    i=$((i + 1))
done

# Validate required argument
if [ -z "$BUMP_TYPE" ]; then
    error "Missing required argument: bump-type (patch|minor|major)"
    exit 1
fi

# Function to compute new version
compute_new_version() {
    local current="$1"
    local bump_type="$2"

    # Extract MAJOR.MINOR.PATCH
    local major minor patch
    IFS='.' read -r major minor patch <<< "$current"

    case "$bump_type" in
        patch)
            patch=$((patch + 1))
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
    esac

    echo "${major}.${minor}.${patch}"
}

# Function to find last tag matching a prefix
find_last_tag() {
    local prefix="$1"
    git tag -l "${prefix}*" 2>/dev/null | sort -V | tail -1 || echo ""
}

# Function to show git log since last tag
show_changelog() {
    local prefix="$1"
    local last_tag
    last_tag=$(find_last_tag "$prefix")

    if [ -z "$last_tag" ]; then
        info "No previous tags found. Showing recent commits:"
        git log --oneline -20
    else
        info "Changes since $last_tag:"
        git log "${last_tag}..HEAD" --oneline
    fi
}

# Global version state (avoids command substitution swallowing stdout)
SCHEMA_OLD=""
SCHEMA_NEW=""
ENGINE_OLD=""
ENGINE_NEW=""

# ============================================================================
# SCHEMA TRACK
# ============================================================================

bump_schema() {
    local dry_run="$1"
    local schema_lib="crates/citum-schema/src/lib.rs"
    local schema_doc="docs/reference/SCHEMA_VERSIONING.md"

    if ! grep -q 'fn default_version()' "$schema_lib"; then
        error "Could not find default_version() in $schema_lib"
        return 1
    fi

    SCHEMA_OLD=$(grep -A1 'fn default_version()' "$schema_lib" | grep -o '"[^"]*"' | tr -d '"' | head -1)
    SCHEMA_NEW=$(compute_new_version "$SCHEMA_OLD" "$BUMP_TYPE")

    info "Schema track:"
    echo "  Current version : $SCHEMA_OLD"
    echo "  New version     : $SCHEMA_NEW"
    echo "  Files to update : $schema_lib, $schema_doc"
    show_changelog "schema-v"

    if [ "$dry_run" = true ]; then
        return 0
    fi

    info "Updating default_version() in $schema_lib"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "/fn default_version() -> String {/,/}/ s/\"$SCHEMA_OLD\"/\"$SCHEMA_NEW\"/" "$schema_lib"
    else
        sed -i "/fn default_version() -> String {/,/}/ s/\"$SCHEMA_OLD\"/\"$SCHEMA_NEW\"/" "$schema_lib"
    fi

    info "Validating schema with cargo test"
    if ! cargo test --quiet --lib 2>&1 | grep -q "test result: ok"; then
        error "Schema validation failed. Reverting changes..."
        git checkout "$schema_lib"
        return 1
    fi
    success "Schema validation passed"

    info "Updating schema changelog in $schema_doc"
    local timestamp entry
    timestamp=$(date +%Y-%m-%d)
    entry="#### schema-v${SCHEMA_NEW} (${timestamp})\n- Schema version bumped to ${SCHEMA_NEW}\n"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "/### Schema Changelog/a\\
\\
$entry" "$schema_doc"
    else
        sed -i "/### Schema Changelog/a\\$entry" "$schema_doc"
    fi
}

# ============================================================================
# ENGINE TRACK
# ============================================================================

bump_engine() {
    local dry_run="$1"
    local cargo_toml="Cargo.toml"

    ENGINE_OLD=$(grep -A5 '^\[workspace.package\]' "$cargo_toml" | grep 'version' | head -1 | grep -o '"[^"]*"' | tr -d '"')
    if [ -z "$ENGINE_OLD" ]; then
        error "Could not find version in [workspace.package] in $cargo_toml"
        return 1
    fi

    ENGINE_NEW=$(compute_new_version "$ENGINE_OLD" "$BUMP_TYPE")

    info "Engine track:"
    echo "  Current version : $ENGINE_OLD"
    echo "  New version     : $ENGINE_NEW"
    echo "  Files to update : $cargo_toml"
    show_changelog "v"

    if [ "$dry_run" = true ]; then
        return 0
    fi

    info "Updating workspace version in $cargo_toml"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \"$ENGINE_OLD\"/version = \"$ENGINE_NEW\"/" "$cargo_toml"
    else
        sed -i "s/^version = \"$ENGINE_OLD\"/version = \"$ENGINE_NEW\"/" "$cargo_toml"
    fi

    info "Validating engine with cargo fmt and clippy"
    if ! cargo fmt --check 2>/dev/null; then
        warn "cargo fmt check had issues, running cargo fmt"
        cargo fmt
    fi
    if ! cargo clippy --all-targets --all-features -- -D warnings 2>&1 | grep -q "^error"; then
        success "Cargo clippy validation passed"
    else
        error "Cargo clippy validation failed. Reverting changes..."
        git checkout "$cargo_toml"
        return 1
    fi
}

# ============================================================================
# MAIN LOGIC
# ============================================================================

main() {
    local all_pass=true

    if [ "$TRACK" = "schema" ] || [ "$TRACK" = "all" ]; then
        bump_schema "$DRY_RUN" || all_pass=false
    fi

    if [ "$TRACK" = "engine" ] || [ "$TRACK" = "all" ]; then
        bump_engine "$DRY_RUN" || all_pass=false
    fi

    if [ "$all_pass" = false ]; then
        error "Bump validation failed"
        exit 1
    fi

    if [ "$DRY_RUN" = true ]; then
        echo ""
        success "Dry-run complete (no changes made)"
        exit 0
    fi

    # Show diff and prompt for commit
    echo ""
    info "Review changes:"
    if [ "$TRACK" = "schema" ] || [ "$TRACK" = "all" ]; then
        git diff crates/citum-schema/src/lib.rs docs/reference/SCHEMA_VERSIONING.md
    fi
    if [ "$TRACK" = "engine" ] || [ "$TRACK" = "all" ]; then
        git diff Cargo.toml
    fi

    echo ""
    read -p "Commit these changes? (y/N) " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        warn "Changes not committed."
        if [ "$TRACK" = "schema" ] || [ "$TRACK" = "all" ]; then
            git checkout crates/citum-schema/src/lib.rs docs/reference/SCHEMA_VERSIONING.md
        fi
        if [ "$TRACK" = "engine" ] || [ "$TRACK" = "all" ]; then
            git checkout Cargo.toml
        fi
        exit 0
    fi

    # Build commit message and tag list
    local commit_msg tag_list=()
    local name_suffix=""
    [ -n "$RELEASE_NAME" ] && name_suffix=" — ${RELEASE_NAME}"

    if [ "$TRACK" = "all" ]; then
        commit_msg="chore: bump versions to v${ENGINE_NEW} / schema-v${SCHEMA_NEW}${name_suffix}

Bumped engine and schema versions from ${ENGINE_OLD} to ${ENGINE_NEW}."
        tag_list=("v${ENGINE_NEW}" "schema-v${SCHEMA_NEW}")
    elif [ "$TRACK" = "schema" ]; then
        commit_msg="chore(schema): bump schema to schema-v${SCHEMA_NEW}${name_suffix}

Schema version bumped from ${SCHEMA_OLD} to ${SCHEMA_NEW}."
        tag_list=("schema-v${SCHEMA_NEW}")
    elif [ "$TRACK" = "engine" ]; then
        commit_msg="chore(engine): bump engine to v${ENGINE_NEW}${name_suffix}

Engine version bumped from ${ENGINE_OLD} to ${ENGINE_NEW}."
        tag_list=("v${ENGINE_NEW}")
    fi

    # Stage and commit
    if [ "$TRACK" = "schema" ] || [ "$TRACK" = "all" ]; then
        git add crates/citum-schema/src/lib.rs docs/reference/SCHEMA_VERSIONING.md
    fi
    if [ "$TRACK" = "engine" ] || [ "$TRACK" = "all" ]; then
        git add Cargo.toml
    fi
    git commit -m "$commit_msg"
    success "Changes committed"

    # Create tags
    for tag in "${tag_list[@]}"; do
        local tag_msg="Version ${tag}${name_suffix}"
        info "Creating git tag: $tag"
        git tag -a "$tag" -m "$tag_msg"
        success "Tag created: $tag"
    done

    echo ""
    success "Version bump complete!"
    info "Next steps:"
    echo "  1. Review the commit and tags"
    echo "  2. Push to remote: git push && git push --tags"
    echo "  3. Create GitHub Release for the tags"
}

main
