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
- **Git-aware awareness** — compare ticket state across branches and worktrees.
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

# Check what another branch is doing
tndm awareness --against branch-a
# → JSON report of added, removed, and diverged tickets

# Keep formatting consistent
tndm fmt --check
```

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
