#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_root="$repo_root/.codex/skills"
codex_home="${CODEX_HOME:-$HOME/.codex}"
target_root="$codex_home/skills"

if [[ ! -d "$source_root" ]]; then
  echo "Skill source directory not found: $source_root" >&2
  exit 1
fi

mkdir -p "$target_root"

for skill_dir in "$source_root"/*; do
  [[ -d "$skill_dir" ]] || continue
  skill_name="$(basename "$skill_dir")"
  target_path="$target_root/$skill_name"

  if [[ -e "$target_path" || -L "$target_path" ]]; then
    if [[ ! -L "$target_path" ]]; then
      echo "Refusing to replace non-symlink target: $target_path" >&2
      exit 1
    fi
    current_target="$(readlink "$target_path")"
    if [[ "$current_target" == "$skill_dir" ]]; then
      continue
    fi
    rm "$target_path"
  fi

  ln -s "$skill_dir" "$target_path"
done

echo "Installed Codex skills into $target_root"
