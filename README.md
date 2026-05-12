# tandem (`tndm`)

> **Git-aware ticket coordination for AI agents in a monorepo.**

Store ticket state in your repository. Work across branches and git worktrees.
No central service. No background process. Just `tndm`.

[![CI](https://github.com/mrclrchtr/tandem/actions/workflows/ci.yml/badge.svg)](https://github.com/mrclrchtr/tandem/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/tandem-cli)](https://crates.io/crates/tandem-cli)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

---

- [Why tandem?](#why-tandem)
- [Project status](#project-status)
- [Quick install](#quick-install)
- [30-second tour](#30-second-tour)
- [Agent plugin](#agent-plugin)
- [Documentation](#documentation)
- [Getting help](#getting-help)
- [Contributing](#contributing)
- [License](#license)

## Why tandem?

When multiple AI agents work in the same repo, they step on each other.
`tandem` gives every agent a shared, deterministic view of what tickets exist,
who is working on what, and how work has diverged across branches — without
leaving Git.

- **Repo-local state** — tickets live in `.tndm/` and travel with your code.
- **Document registry** — ticket-owned markdown files registered in metadata; agents edit at the file path, not through CLI strings.
- **Fingerprint verification** — SHA-256 fingerprints in `state.toml` ensure `tndm fmt --check` catches stale content after file edits.
- **Git-aware awareness** — `tndm awareness --against <ref>` compares ticket states across branches and worktrees, including document fingerprint diffs.
- **Deterministic format** — canonical TOML + `tndm fmt --check` for clean diffs.
- **Agent-first, human-friendly** — built for autonomous agents; humans review via `tndm ticket show` with colored output.
- **Zero infrastructure** — no database, no cloud service, no LLM required.

## Project status

`tandem` is **pre-1.0, active development**. The core CLI and awareness features
work today. The API and on-disk format are stabilizing but may evolve before 1.0.
See [`docs/vision.md`](docs/vision.md) for scope and roadmap.

## Quick install

```sh
# Prebuilt binary (macOS / Linux)
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/mrclrchtr/tandem/releases/latest/download/tndm-installer.sh | sh

# Homebrew
brew install mrclrchtr/tap/tandem-cli

# From source
cargo install --path crates/tandem-cli
```

## 30-second tour

```sh
# Create a ticket
tndm ticket create "Refactor auth module"
# → TNDM-A1B2C3

# Set status and priority during creation
tndm ticket create "Fix login bug" --type bug --priority p1 --status in_progress
# → TNDM-6F2E1A

# Update status, add tags
tndm ticket update TNDM-A1B2C3 --status in_progress --tags auth,security

# Register a document for detailed content (preferred over large CLI strings)
tndm ticket doc create TNDM-A1B2C3 plan
# → .tndm/tickets/TNDM-A1B2C3/docs/plan.md
# (edit that file with your editor, then sync fingerprints)
tndm ticket sync TNDM-A1B2C3

# View a formatted ticket
tndm ticket show TNDM-A1B2C3
# →
#   TNDM-A1B2C3 · Refactor auth module
#   ──────────────────────────────────────────────
#
#     Status      · in_progress   ← blue in terminal
#     Priority    · p2
#     Type        · task
#
#     Updated     · 2026-05-03T21:28:10Z (rev 1)
#
#   ──────────────────────────────────────────────
#   Content
#   ──────────────────────────────────────────────
#   ## Context
#   ...

# List all active tickets
tndm ticket list
# → ID              STATUS        PRIORITY  TITLE
#   TNDM-A1B2C3     in_progress   p2        Refactor auth module
#   TNDM-6F2E1A     in_progress   p1        Fix login bug

# Show all tickets including done
tndm ticket list --all

# Output as JSON for agent consumption
tndm ticket show TNDM-A1B2C3 --json
# → {"meta":{...},"state":{...},"content":"..."}

# Check what another branch is working on
tndm awareness --against branch-a
# → JSON report with added, removed, and diverged tickets

# Keep formatting consistent in CI
tndm fmt --check
```

**Human-friendly output.** `tndm ticket show` formats tickets with color-coded status
(done → green, in_progress → blue, blocked → red, todo → yellow),
aligned fields, and rendered Markdown content — headings, bold, italic,
code blocks, lists, and blockquotes are all styled in the terminal.
Colors disable automatically when output is piped.

**Agent-friendly JSON.** Append `--json` to any command for deterministic structured
output — no parsing human-readable text required.


## Agent plugin

Load the `tndm` plugin into Claude Code, Codex, or any skills.sh-compatible agent so agents
create tickets, update status, and run awareness checks automatically.

### Via skills.sh (any agent)

```sh
npx skills add mrclrchtr/tandem --skill ticket --skill awareness
```

This installs the `ticket` and `awareness` skills locally so your agent will create tickets,
update status, and run awareness checks automatically.

### Claude Code

```sh
claude --plugin-dir ./plugins/tndm
```

### Codex

See [`plugins/tndm/README.md`](plugins/tndm/README.md) for marketplace setup instructions.

### PI (coding agent)

[`plugins/supi-flow/`](plugins/supi-flow/) is a **PI extension** that adds a spec-driven workflow (brainstorm → plan → apply → archive) coupled to TNDM ticket coordination. It ships with 6 auto-discovered skills and 5 custom tools (`supi_flow_start`, `supi_flow_plan`, `supi_flow_complete_task`, `supi_flow_close`, `supi_tndm_cli`).

```bash
pi install npm:@mrclrchtr/supi-flow
```

## Documentation

| Doc | What you'll find |
|-----|------------------|
| [`docs/vision.md`](docs/vision.md) | Product goals, core workflow, V1 scope |
| [`docs/architecture.md`](docs/architecture.md) | Crate structure, dependency rules, enforcement |
| [`docs/decisions.md`](docs/decisions.md) | Design rationale and trade-offs |
| [`docs/references.md`](docs/references.md) | Competitive analysis and related projects |
| [`CHANGELOG.md`](CHANGELOG.md) | Release history |

## Getting help

- **Bug reports / feature requests** — open a [GitHub issue](https://github.com/mrclrchtr/tandem/issues)
- **Questions / discussion** — start a [GitHub discussion](https://github.com/mrclrchtr/tandem/discussions)
- **CLI help** — `tndm --help` for full command reference

## Contributing

`tandem` is a Rust workspace managed with `mise`.

```sh
mise install        # install toolchain
mise run test       # run the test suite
mise run check      # fmt + compile + arch + clippy
```

See [`CLAUDE.md`](CLAUDE.md) for development conventions and [`docs/releasing.md`](docs/releasing.md) for the release process.

## License

`tandem` is licensed under [Apache 2.0](LICENSE).
