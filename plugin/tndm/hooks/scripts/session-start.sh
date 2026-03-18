#!/bin/bash
# Injects a tndm advisory at session start, but only in repositories that use tndm.
# Silently exits in repos without a .tndm/ directory.

set -euo pipefail

TNDM_DIR="${CLAUDE_PROJECT_DIR:-$(pwd)}/.tndm"

if [ ! -d "$TNDM_DIR" ]; then
  exit 0
fi

cat <<'EOF'
{
  "systemMessage": "This repository uses tndm for ticket coordination. When starting any task or development work: (1) create a ticket with `tndm ticket create \"<title>\"` (prints just the ticket ID), (2) immediately update its status to in_progress, (3) run `tndm awareness --against <ref> --json` before starting work that may overlap with another branch. Keep ticket status current as work progresses. Run `tndm fmt` after every ticket mutation. Use the /tndm:ticket and /tndm:awareness skills for guided workflows."
}
EOF
