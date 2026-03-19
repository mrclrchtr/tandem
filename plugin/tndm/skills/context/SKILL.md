---
name: tndm context
description: >
  This skill MUST be used when an agent starts any development activity that should be
  tracked. It MUST also be used when the conversation mentions "ticket", "tndm", "TNDM-*",
  "track this work", "create a ticket", "what tickets are open", "coordination", "worktree",
  "awareness", "branch coordination", "what changed on branch", or "in_progress".
user-invocable: false
version: 0.1.0
---

# tndm — Ticket Coordination for AI Agents

tndm is a git-aware ticket coordination system for AI agents in a monorepo. It stores ticket state
in the repository (no central service required), works across branches and git worktrees, and lets
agents discover what other agents have changed via structured awareness output.

**Agent-first design**: agents create, update, and query tickets through a deterministic CLI.
Humans use `tndm` for oversight. Every ticket lives as files in `.tndm/tickets/<ID>/`.

## Core Protocol

Follow this workflow for every development task:

### 1. Start of work — create a ticket

Before writing code or making changes:

```sh
tndm ticket create "Brief title describing the task"
```

Note the returned ticket ID (format: `TNDM-XXXXXX`). Immediately update status:

```sh
tndm ticket update <ID> --status in_progress
```

### 2. Before branching or starting work that may overlap

Run awareness against the target branch before starting to detect conflicts with other agents:

```sh
tndm awareness --against <branch-or-ref> --json
```

Inspect `diverged` entries for tickets that exist on both refs with differing fields. Adjust the
plan to avoid conflicting changes before proceeding.

### 3. During work — update status as it changes

Keep ticket status current as work progresses:

```sh
# When blocked — document the reason via heredoc (do not create temporary files):
tndm ticket update <ID> --status blocked <<'EOF'
Blocked: waiting for PR #42 review
EOF

# When unblocked and resuming:
tndm ticket update <ID> --status in_progress

# See references/command-reference.md for all field-update patterns
```

### 4. When work is complete — mark done

```sh
tndm ticket update <ID> --status done
```

Marking `done` is the inter-agent coordination signal. Other agents and awareness checks rely on
this to know the work is finished and safe to build on.

### 5. After ticket creation or status change — commit immediately

Ticket creation and status updates (`in_progress`, `blocked`, `done`) are coordination signals.
Other agents can only see them once committed. Always commit these changes right away (`git add .tndm/ && git commit`).

## Ticket Fields Quick Reference

| Field          | Valid values                                    | Default  |
|----------------|-------------------------------------------------|----------|
| `--status`     | `todo` `in_progress` `blocked` `done`           | `todo`   |
| `--priority`   | `p0` `p1` `p2` `p3` `p4`                       | `p2`     |
| `--type`       | `task` `bug` `feature` `chore` `epic`           | `task`   |
| `--tags`       | comma-separated strings (replaces existing)     | —        |
| `--depends-on` | comma-separated ticket IDs (replaces existing)  | —        |

All `tndm ticket` subcommands accept `--json` for machine-readable output. Prefer the default text output for mutations (just prints the ticket ID) to save tokens.

## Key Rules

- **Always create a ticket before starting work.** Skipping this for "quick" tasks is not permitted.
- **Keep status current.** Other agents rely on status to understand what is in flight.
- **Run awareness before branching.** Discover coordination needs before writing code.
- **Use `--json`** on read commands (`show`, `list`, `awareness`) when parsing output for decisions. Mutations (`create`, `update`) already print just the ticket ID by default — skip `--json` to save tokens.
- **Commit ticket creation and status changes immediately.** These are coordination signals — other agents can only see them once committed. Always `git add .tndm/ && git commit` right after creating a ticket or changing its status.
- **Never hardcode ticket IDs** in code. Reference them in commit messages and ticket content only.

## Awareness Output Structure

`tndm awareness --against <ref> --json` returns:

```json
{
  "added_current":  [ /* tickets that exist only on the current branch */ ],
  "added_against":  [ /* tickets that exist only on <ref> */ ],
  "diverged":       [
    {
      "id": "TNDM-XXXXXX",
      "current": { /* ticket snapshot on current branch */ },
      "against":  { /* ticket snapshot on <ref> */ },
      "diff": {
        "status":     { "current": "in_progress", "against": "done" },
        "priority":   { "current": "p1",           "against": "p2"  }
      }
    }
  ]
}
```

Use `added_against` to detect work in flight on the other branch. Use `diverged.diff` to identify
specific field conflicts before merging.

## Additional Resources

For complete command syntax, all flags, enum values, ticket file layout, and repository
configuration options, see:

- **`references/command-reference.md`** — full flag reference, all subcommands, enum values,
  example invocations, ticket file structure, and `.tndm/config.toml` options
