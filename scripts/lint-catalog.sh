#!/usr/bin/env bash
# Lint Market Forge algorithm-catalog entries against docs/standards.md §2.
#
# Every file under docs/catalog/**/*.md (except _index.md / README.md) must contain the
# required section markers and a mermaid diagram. Exits non-zero if any entry is missing one.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CATALOG="$ROOT/docs/catalog"

if [[ ! -d "$CATALOG" ]]; then
  echo "lint-catalog: no docs/catalog/ directory yet — nothing to lint."
  exit 0
fi

# Required literal markers (see docs/standards.md §2).
REQUIRED=(
  '```mermaid'
  '**What it is.**'
  '**When to pick this.**'
  '**When NOT to pick this.**'
  '**Real venue.**'
  '**Recommended crate.**'
)

fail=0
count=0
while IFS= read -r -d '' file; do
  base="$(basename "$file")"
  case "$base" in
    _index.md | README.md) continue ;;
  esac
  count=$((count + 1))

  # Must start with a level-1 title.
  if ! head -n 20 "$file" | grep -qE '^# .+'; then
    echo "FAIL  $file — missing level-1 '# Title'"
    fail=1
  fi

  for marker in "${REQUIRED[@]}"; do
    if ! grep -qF "$marker" "$file"; then
      echo "FAIL  $file — missing marker: $marker"
      fail=1
    fi
  done

  # Word-count guard on the prose (≤300 words is the style cap; warn past 350 to allow
  # diagram source + table tokens).
  words="$(wc -w < "$file" | tr -d ' ')"
  if [[ "$words" -gt 600 ]]; then
    echo "WARN  $file — $words words (prose should be ≤300; trim or split)"
  fi
done < <(find "$CATALOG" -name '*.md' -type f -print0)

echo "lint-catalog: checked $count catalog entries."
if [[ "$fail" -ne 0 ]]; then
  echo "lint-catalog: FAILED — fix the entries above."
  exit 1
fi
echo "lint-catalog: OK"
