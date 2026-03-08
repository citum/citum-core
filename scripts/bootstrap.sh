#!/bin/bash

set -euo pipefail

MODE="${1:-minimal}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

usage() {
    cat <<EOF
Usage: ./scripts/bootstrap.sh [minimal|full]

minimal  Install local script dependencies and keep corpus submodules optional.
full     Install script dependencies and shallow-fetch styles-legacy plus tests/csl-test-suite.
EOF
}

case "${MODE}" in
    minimal|full)
        ;;
    -h|--help|help)
        usage
        exit 0
        ;;
    *)
        usage >&2
        exit 1
        ;;
esac

cd "${PROJECT_ROOT}"

echo "Installing Node dependencies for scripts/ ..."
npm install --prefix scripts

if [ "${MODE}" = "full" ]; then
    echo "Initializing optional corpora with shallow submodule checkout ..."
    git submodule update --init --depth 1 styles-legacy tests/csl-test-suite
fi

cat <<EOF

Bootstrap complete (${MODE}).
Use ./scripts/dev-env.sh <command> to keep CARGO_TARGET_DIR outside the repo.
Examples:
  ./scripts/dev-env.sh cargo build --workspace
  ./scripts/dev-env.sh cargo test --workspace
EOF
