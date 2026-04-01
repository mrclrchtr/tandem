# tndm Agent Plugin

Instructs AI agents to use the `tndm` ticket coordination CLI for task tracking and branch
awareness in a monorepo.

This repository ships the `tndm` plugin from a single canonical directory:
- repo source: `plugins/tndm`
- Claude Code packaging: `plugins/tndm/.claude-plugin`
- Codex packaging: `plugins/tndm/.codex-plugin`

## What It Does

Without this plugin, agents have no knowledge of `tndm` and will skip ticket creation, never
update status, and ignore the awareness workflow.

With this plugin loaded, agents:
- Only create or update tickets when the user explicitly asks to track work or references a ticket
- Keep ticket status current (`in_progress` -> `blocked` -> `done`)
- Run `tndm awareness` before branching to detect conflicts with other agents
- Commit ticket changes immediately so other agents can see them

## Components

| Component | Type | Purpose |
|---|---|---|
| `skills/ticket` | Skill + slash command | `/tndm:ticket create\|update\|show\|list` — workflow protocol + full ticket lifecycle |
| `skills/awareness` | Skill + slash command | `/tndm:awareness <ref>` — checks what changed on another branch |
| `hooks/hooks.json` | Claude hook config | Injects open tickets as context on start and reminds agents on stop |
| `hooks.json` | Codex hook config | Registers the same ticket checks for Codex plugin installs |

## Usage

### Claude Code

Load per session (development / ad-hoc):

```sh
claude --plugin-dir ./plugins/tndm
```

Install as a project-scoped plugin once a marketplace is configured, or add the `--plugin-dir`
flag to your shell alias / `mise` task.

Verify it loaded:

```
/help
```

Skills `/tndm:awareness` and `/tndm:ticket` should appear in the output.

### Codex

Current Codex support is through a personal marketplace.

Install layout:
- plugin directory: `~/.codex/plugins/tndm`
- marketplace file: `~/.agents/plugins/marketplace.json`

Marketplace entry:

```json
{
  "name": "tndm",
  "source": {
    "source": "local",
    "path": "./.codex/plugins/tndm"
  },
  "policy": {
    "installation": "AVAILABLE",
    "authentication": "ON_INSTALL"
  },
  "category": "Coding"
}
```

Notes:
- Restart Codex after adding or updating the personal marketplace.
- Install `tndm` from the personal marketplace in the Codex plugin directory.
- Keep the plugin self-contained. Nested symlinks inside the plugin folder are not reliable for
  Codex installs because Codex loads the installed cached copy under
  `~/.codex/plugins/cache/...`.

## Requirements

- Claude Code >= 1.0.33
- Codex with plugin support enabled
- `tndm` CLI available in `PATH` (built with `cargo build` or installed via the repo's `tndm-dev` wrapper)

## Slash Commands

| Command | Description |
|---|---|
| `/tndm:awareness <ref>` | Run awareness check against a git ref |
| `/tndm:ticket create <title>` | Create a new ticket |
| `/tndm:ticket update <ID> [flags]` | Update ticket fields |
| `/tndm:ticket show <ID>` | Show a single ticket |
| `/tndm:ticket list` | List tickets (done hidden by default; use `--all` to include) |
