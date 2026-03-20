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
- Commit ticket changes immediately so other agents can see them

## Components

| Component | Type | Purpose |
|---|---|---|
| `skills/ticket` | Skill + slash command | `/tndm:ticket create\|update\|show\|list` — workflow protocol + full ticket lifecycle |
| `skills/awareness` | Skill + slash command | `/tndm:awareness <ref>` — checks what changed on another branch |
| `hooks/hooks.json` (SessionStart) | Hook | Injects open tickets as context so agents are aware from the start |
| `hooks/hooks.json` (Stop/SubagentStop) | Hook | Lists open tickets in the transcript as a reminder when stopping |

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
| `/tndm:ticket list` | List tickets (done hidden by default; use `--all` to include) |
