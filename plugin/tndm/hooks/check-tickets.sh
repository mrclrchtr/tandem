#!/bin/bash
set -euo pipefail

# List open tickets (excludes "done" by default).
# Used by SessionStart (context injection) and Stop (reminder).
# Exits 0 always — informational only, never blocks.

output=$(tndm ticket list --json 2>/dev/null) || exit 0

count=$(echo "$output" | jq '.tickets | length')

if [ "$count" -eq 0 ]; then
  exit 0
fi

echo "Open tndm tickets ($count):"
echo "$output" | jq -r '.tickets[] | "  \(.id)  \(.status | ascii_upcase)  \(.title)"'
