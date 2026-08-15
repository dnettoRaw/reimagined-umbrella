#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
violations=0

while IFS= read -r file; do
  case "$file" in
    */target/*|*/tests/*|*/examples/*|*/templates/*|*_tests.rs|*/src/tests.rs)
      continue
      ;;
  esac
  lines="$(wc -l <"$file" | tr -d ' ')"
  if (( lines > 500 )); then
    printf 'production Rust module exceeds 500 lines: %s (%s)\n' "$file" "$lines" >&2
    violations=1
  fi
done < <(find "$ROOT_DIR/proexel" "$ROOT_DIR/core/AppCore-Runtime" -type f -name '*.rs' | sort)

while IFS= read -r file; do
  if ! awk '
    BEGIN { in_reexport = 0; invalid = 0 }
    /^[[:space:]]*$/ || /^[[:space:]]*\/\// { next }
    in_reexport {
      if ($0 ~ /;[[:space:]]*$/) in_reexport = 0
      next
    }
    /^[[:space:]]*pub[[:space:]]+mod[[:space:]]+[A-Za-z0-9_]+[[:space:]]*;[[:space:]]*$/ { next }
    /^[[:space:]]*pub[[:space:]]+use[[:space:]]+/ {
      if ($0 !~ /;[[:space:]]*$/) in_reexport = 1
      next
    }
    { invalid = 1 }
    END { exit invalid }
  ' "$file"; then
    printf 'mod.rs contains implementation or unsupported declarations: %s\n' "$file" >&2
    violations=1
  fi
done < <(find "$ROOT_DIR/proexel" "$ROOT_DIR/core/AppCore-Runtime" -type f -name mod.rs | sort)

if (( violations != 0 )); then
  exit 1
fi

printf 'Rust structure check passed.\n'
