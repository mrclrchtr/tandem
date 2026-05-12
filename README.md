# tandem (`tndm`)

> **Git-aware ticket coordination for AI agents in a monorepo.**

Store ticket state in your repository. Work across branches and git worktrees.
No central service. No background process. Just `tndm`.

---

## Why tandem?

When multiple AI agents work in the same repo, they step on each other.
`tandem` gives every agent a shared, deterministic view of what tickets exist,
who is working on what, and how work has diverged across branches — without
leaving Git.

- **Repo-local state** — tickets live in `.tndm/` and travel with your code.
- **Document registry** — ticket-owned markdown files registered in metadata; agents edit at the file path, not through CLI strings.
- **Fingerprint verification** — SHA-256 fingerprints in `state.toml` ensure `tndm fmt --check` catches stale content after file edits.
- **Git-aware awareness** — compare ticket state across branches and worktrees, including document fingerprint diffs.
- **Deterministic format** — canonical TOML + `tndm fmt --check` for clean diffs.
- **Agent-first, human-friendly** — built for autonomous agents; humans review via CLI.
- **Zero infrastructure** — no database, no cloud service, no LLM required.

## Quick install

```sh
# Prebuilt binary (macOS / Linux)
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/mrclrchtr/tandem/releases/latest/download/tndm-installer.sh | sh

# Homebrew
brew install mrclrchtr/tap/tndm

# From source
cargo install --path crates/tandem-cli
```

## 30-second tour

```sh
# Create a ticket
tndm ticket create "Refactor auth module"
# → TNDM-A1B2C3

# Update status
tndm ticket update TNDM-A1B2C3 --status in_progress

# Register a document for detailed content (preferred over large CLI strings)
tndm ticket doc create TNDM-A1B2C3 plan
# → .tndm/tickets/TNDM-A1B2C3/docs/plan.md
# (edit that file with your editor, then sync fingerprints)
tndm ticket sync TNDM-A1B2C3

# View a ticket with its rich, formatted output
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

# Check what another branch is doing
tndm awareness --against branch-a
# → JSON report of added, removed, and diverged tickets

# Keep formatting consistent
tndm fmt --check
```

**Human-friendly output.** `tndm ticket show` formats tickets with color-coded status
(done → green, in_progress → blue, blocked → red, todo → yellow),
aligned fields, and rendered Markdown content — headings, bold, italic,
code blocks, lists, and blockquotes are all styled in the terminal.
Colors disable automatically when output is piped.

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

## Contributing

`tandem` is a Rust workspace managed with `mise`.

```sh
mise install        # install toolchain
mise run test       # run the test suite
mise run check      # fmt + compile + arch + clippy
```

See [`CLAUDE.md`](CLAUDE.md) for development conventions and [`docs/releasing.md`](docs/releasing.md) for the release process.

## License

Apache-2.0
