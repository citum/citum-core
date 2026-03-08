#!/bin/bash

DATASET_GUARD_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATASET_BOOTSTRAP_COMMAND="./scripts/bootstrap.sh full"

dataset_guard_message() {
    local context="$1"
    shift
    local summary="$*"
    cat >&2 <<EOF
${context} requires optional local datasets that are not checked out: ${summary}.
Use the lean daily setup for normal Rust work, or fetch the corpora on demand with:
  ${DATASET_BOOTSTRAP_COMMAND}
EOF
}

require_dataset_dir() {
    local relative_path="$1"
    local label="$2"
    local context="$3"
    local absolute_path

    if [[ "${relative_path}" = /* ]]; then
        absolute_path="${relative_path}"
    else
        absolute_path="${DATASET_GUARD_ROOT}/${relative_path}"
    fi

    if [ ! -d "${absolute_path}" ]; then
        dataset_guard_message "${context}" "${label} (${relative_path})"
        exit 2
    fi
}

require_dataset_file() {
    local relative_path="$1"
    local label="$2"
    local context="$3"
    local absolute_path
    local summary_path="${relative_path%/*}"

    if [[ "${relative_path}" = /* ]]; then
        absolute_path="${relative_path}"
        summary_path="$(basename "$(dirname "${relative_path}")")"
    else
        absolute_path="${DATASET_GUARD_ROOT}/${relative_path}"
    fi

    if [ ! -f "${absolute_path}" ]; then
        dataset_guard_message "${context}" "${label} (${summary_path})"
        exit 2
    fi
}
