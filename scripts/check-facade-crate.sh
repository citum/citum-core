#!/usr/bin/env bash

set -euo pipefail

crate_dir="crates/citum-schema"
src_dir="$crate_dir/src"
lib_rs="$src_dir/lib.rs"

source_files=()
while IFS= read -r file; do
  source_files+=("$file")
done < <(cd "$src_dir" && find . -type f | sort)

if [[ "${#source_files[@]}" -ne 1 || "${source_files[0]}" != "./lib.rs" ]]; then
  echo "error: $crate_dir must remain a facade crate with only src/lib.rs" >&2
  printf 'unexpected files:\n' >&2
  printf '  %s\n' "${source_files[@]}" >&2
  exit 1
fi

if ! rg -q '^pub use citum_schema_style::\*;$' "$lib_rs"; then
  echo "error: $lib_rs must re-export citum_schema_style" >&2
  exit 1
fi

if ! rg -q '^pub mod data \{$' "$lib_rs"; then
  echo "error: $lib_rs must expose the data facade module" >&2
  exit 1
fi

if ! rg -q '^    pub use citum_schema_data::\*;$' "$lib_rs"; then
  echo "error: $lib_rs must re-export citum_schema_data inside pub mod data" >&2
  exit 1
fi

while IFS= read -r entry; do
  line=${entry%%:*}
  text=${entry#*:}
  if [[ "$text" != "pub mod data {" ]]; then
    echo "error: unexpected local module declaration in $lib_rs:$line -> $text" >&2
    exit 1
  fi
done < <(rg -n '^\s*(pub\s+)?mod\s+' "$lib_rs" || true)
