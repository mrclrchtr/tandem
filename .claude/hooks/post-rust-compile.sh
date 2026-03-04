#!/usr/bin/env bash
set -euo pipefail

payload="$(cat)"

file_path="$(python3 - "$payload" <<'PY'
import json
import os
import sys

try:
    event = json.loads(sys.argv[1])
except Exception:
    print("")
    raise SystemExit(0)

tool_input = event.get("tool_input") or {}
path = tool_input.get("file_path") or ""
cwd = event.get("cwd") or os.environ.get("CLAUDE_PROJECT_DIR") or ""

if path and not os.path.isabs(path) and cwd:
    path = os.path.normpath(os.path.join(cwd, path))

print(path)
PY
)"

case "$file_path" in
  *.rs) ;;
  *) exit 0 ;;
esac

echo "[hook] Rust file edited: $file_path" >&2
mise run compile
