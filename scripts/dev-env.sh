#!/bin/bash

set -euo pipefail

CACHE_ROOT="${XDG_CACHE_HOME:-$HOME/.cache}/citum-core"
TARGET_DIR="${CITUM_TARGET_DIR:-${CACHE_ROOT}/target}"

mkdir -p "${TARGET_DIR}"
export CARGO_TARGET_DIR="${TARGET_DIR}"

if [ "$#" -eq 0 ]; then
    echo "Launching shell with CARGO_TARGET_DIR=${CARGO_TARGET_DIR}"
    exec "${SHELL:-/bin/bash}" -i
fi

exec "$@"
