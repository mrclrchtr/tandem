#!/usr/bin/env bash
set -euo pipefail

payload="$(cat)"

python3 - "$payload" <<'PY'
import json
import os
import sys

try:
    event = json.loads(sys.argv[1])
except Exception:
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow"
        }
    }))
    raise SystemExit(0)

tool_input = event.get("tool_input") or {}
path = tool_input.get("file_path") or ""
cwd = event.get("cwd") or os.environ.get("CLAUDE_PROJECT_DIR") or ""

if path and not os.path.isabs(path) and cwd:
    path = os.path.normpath(os.path.join(cwd, path))

norm = path.replace("\\", "/")

protected_exact = {
    "/hk.pkl",
    "/mise.toml",
    "/crates/xtask/src/main.rs",
}
is_protected = any(norm.endswith(p) for p in protected_exact) or "/.github/workflows/" in norm

if is_protected:
    out = {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "ask",
            "permissionDecisionReason": "Protected policy file edit requires explicit confirmation."
        }
    }
else:
    out = {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow"
        }
    }

print(json.dumps(out))
PY
