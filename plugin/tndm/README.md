# tndm Claude Plugin

Instructs AI agents to use the `tndm` ticket coordination CLI for task tracking and branch
awareness in a monorepo.

## What It Does

Without this plugin, agents have no knowledge of `tndm` and will skip ticket creation, never
update status, and ignore the awareness workflow.

With this plugin loaded, agents:
- Automatically create a ticket before starting any development task
- Keep ticket status current (`in_progress` → `blocked` → `done`)
- Run `tndm awareness` before branching to detect conflicts with other agents
- Receive a session-start reminder in tndm repositories

## Components

| Component | Type | Purpose |
|---|---|---|
| `skills/context` | Background skill (auto-loaded) | Full tndm workflow knowledge — Claude loads this when starting tasks or when tndm context is detected |
| `skills/awareness` | Skill + slash command | `/tndm:awareness <ref>` — checks what changed on another branch |
| `skills/ticket` | Skill + slash command | `/tndm:ticket create\|update\|show\|list` — full ticket lifecycle |
| `hooks/session-start.sh` | SessionStart hook | Injects tndm advisory at session start (only in repos with `.tndm/`) |
| `hooks/hooks.json` (Stop) | Stop hook | Reminds agents to update ticket status when finishing work |

## Usage

### Load per session (development / ad-hoc)

```sh
claude --plugin-dir ./plugin/tndm
```

### Load for the project (all sessions)

Install as a project-scoped plugin once a marketplace is configured, or add the `--plugin-dir`
flag to your shell alias / `mise` task.

### Verify it loaded

```
/help
```

Skills `/tndm:awareness` and `/tndm:ticket` should appear in the output.

## Requirements

- Claude Code ≥ 1.0.33
- `tndm` CLI available in `PATH` (built with `cargo build` or installed via the repo's `tndm-dev` wrapper)

## Slash Commands

| Command | Description |
|---|---|
| `/tndm:awareness <ref>` | Run awareness check against a git ref |
| `/tndm:ticket create <title>` | Create a new ticket |
| `/tndm:ticket update <ID> [flags]` | Update ticket fields |
| `/tndm:ticket show <ID>` | Show a single ticket |
| `/tndm:ticket list` | List all tickets |
