# tandem (`tndm`)

> **Git-aware ticket coordination for AI agents in a monorepo.**

Store ticket state in your repository. Coordinate work across branches and git worktrees.
No central service. No background process. Just `tndm`.

[![CI](https://github.com/mrclrchtr/tandem/actions/workflows/ci.yml/badge.svg)](https://github.com/mrclrchtr/tandem/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/tandem-cli)](https://crates.io/crates/tandem-cli)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

---

- [Why tandem?](#why-tandem)
- [Core capabilities](#core-capabilities)
- [Quick install](#quick-install)
- [CLI design](#cli-design)
- [On-disk ticket model](#on-disk-ticket-model)
- [Quick tour](#quick-tour)
- [JSON / API design](#json--api-design)
- [Architecture at a glance](#architecture-at-a-glance)
- [PI extension](#pi-extension)
- [Project status](#project-status)
- [Documentation](#documentation)
- [Getting help](#getting-help)
- [Contributing](#contributing)
- [License](#license)

## Why tandem?

When multiple AI agents work in the same repo, they step on each other.
`tandem` gives every agent a shared, deterministic view of what tickets exist,
what each ticket contains, and how work has diverged across branches or worktrees
without leaving Git.

The public interface today is the **`tndm` CLI** plus its structured JSON output.
The Rust workspace crates are layered internal implementation modules, not a separate
stable end-user API.

## Core capabilities

`tandem` currently provides:

- **Repo-local ticket lifecycle** — create, list, show, update, and organize tickets under `.tndm/`.
- **Structured ticket metadata** — status, priority, type, effort, dependencies, tags, revision, and timestamps.
- **Document registry** — tickets own registered markdown documents, not just one blob of inline text.
- **Freshness verification** — SHA-256 document fingerprints in `state.toml`; `tndm ticket sync` and `tndm fmt --check` catch stale ticket docs.
- **Task manifests** — store ordered tasks directly in `state.toml` with per-task files, verification notes, and optional linked detail docs.
- **Task detail docs** — `tndm ticket task detail ensure` creates and links canonical `tasks/task-XX.md` docs for existing tasks.
- **Git-aware awareness** — compare the current working tree against another ref or worktree with `tndm awareness --against <ref>`.
- **Deterministic CLI + JSON** — human-readable terminal output for oversight and schema-versioned JSON for agents.
- **Low operational overhead** — no database, no daemon, no web service, no LLM dependency.
- **Repo-local defaults** — optional `.tndm/config.toml` config for ID prefix generation and default ticket content templates.

## Quick install

```sh
# Prebuilt binary (macOS / Linux)
curl -LsSf \
  https://github.com/mrclrchtr/tandem/releases/latest/download/tndm-installer.sh | sh

# Homebrew
brew install mrclrchtr/tap/tandem-cli

# From source
cargo install --path crates/tandem-cli
```

## CLI design

Top-level commands:

| Command | Purpose |
|---|---|
| `tndm fmt` | Canonicalize `.tndm/` files; use `--check` in CI or pre-commit style workflows |
| `tndm ticket ...` | Create, inspect, update, sync, and organize tickets |
| `tndm awareness --against <ref>` | Compare the current ticket snapshot against another git ref |

Ticket subcommands:

| Command | Purpose |
|---|---|
| `tndm ticket create` | Create a ticket with optional `--status`, `--priority`, `--type`, `--effort`, `--tags`, `--depends-on`, `--content`, or `--content-file` |
| `tndm ticket show` | Render a formatted ticket for humans or emit JSON with `--json` |
| `tndm ticket list` | List active tickets; supports `--all` and `--definition ready\|questions\|unknown` |
| `tndm ticket update` | Update metadata, content, tags, dependencies, and effort |
| `tndm ticket doc create` | Create and register a new ticket-local markdown document |
| `tndm ticket sync` | Recompute registered document fingerprints after file edits |
| `tndm ticket task ...` | Manage task manifests stored in `state.toml` |

Task subcommands:

| Command | Purpose |
|---|---|
| `tndm ticket task add` | Add a numbered task with optional files, verification command, notes, and linked detail path |
| `tndm ticket task list` | List tasks in table form or as JSON |
| `tndm ticket task complete` | Mark a task as done |
| `tndm ticket task remove` | Remove a task |
| `tndm ticket task edit` | Change task title, files, verification, notes, or detail linkage |
| `tndm ticket task set` | Bulk-replace the entire task list from a JSON array |
| `tndm ticket task detail ensure` | Ensure the canonical `tasks/task-XX.md` detail doc exists and is linked |
| `tndm ticket task detail clear` | Detach a task detail doc reference without deleting the file |

Design notes from the current source:

- Most end-user commands support `--json` for deterministic machine consumption.
- New ticket content can come from `--content-file`, `--content`, stdin, or the configured/default markdown template.
- `.tndm/config.toml` can set `[id].prefix`, which drives generated IDs and bare-ID normalization for several commands.
- `tndm ticket show` is human-first in a TTY: it renders Markdown content, colors status, and syntax-highlights fenced code blocks when possible.

## On-disk ticket model

Each ticket lives in its own directory:

```text
.tndm/
├── config.toml                  # optional repo-wide tandem config
└── tickets/
    └── TNDM-A1B2C3/
        ├── meta.toml            # stable metadata
        ├── state.toml           # volatile state + task manifest
        ├── content.md           # default ticket body (registered as document "content")
        ├── plan.md              # optional registered document
        └── tasks/
            └── task-01.md       # optional canonical task detail doc
```

The source splits ticket data deliberately:

- **`meta.toml`** stores relatively stable metadata such as `id`, `title`, `type`, `priority`, `effort`, `depends_on`, `tags`, and the registered `documents` list.
- **`state.toml`** stores more volatile state such as `status`, `updated_at`, `revision`, `document_fingerprints`, and `tasks`.
- **`content.md`** is the default markdown document for a ticket and is automatically registered as document `content`.

Example optional config:

```toml
[id]
prefix = "TNDM"

[templates]
content = """
## Context

...
"""
```

Document registry rules that matter in practice:

- Additional docs should be created through `tndm ticket doc create`, not by inventing unregistered files by hand.
- Registered doc paths are ticket-relative and validated; absolute paths and `..` traversal are rejected.
- After editing any registered document on disk, run `tndm ticket sync <ID>`.
- `tndm fmt --check` fails when canonical formatting or document fingerprints drift.
- Task detail docs are manifest-first: tasks live in `state.toml`, and detail docs are optional attachments linked to existing tasks.

## Quick tour

```sh
# Create a ticket with structured metadata
tndm ticket create "Refresh README overview" \
  --type chore \
  --priority p2 \
  --status in_progress \
  --tags docs,definition:ready \
  --effort s
# → TNDM-A1B2C3

# Add a registered plan document and edit it with your normal editor
tndm ticket doc create TNDM-A1B2C3 plan
# → .tndm/tickets/TNDM-A1B2C3/plan.md

# Add an executable task manifest entry
tndm ticket task add TNDM-A1B2C3 \
  --title "Rewrite README capabilities section" \
  --file README.md \
  --verification "manual review against --help output"

# If the task needs its own attachment, ensure the canonical detail doc exists
tndm ticket task detail ensure TNDM-A1B2C3 1
# → .tndm/tickets/TNDM-A1B2C3/tasks/task-01.md

# After editing any registered doc file on disk, refresh fingerprints
tndm ticket sync TNDM-A1B2C3

# Review the ticket for humans or machines
tndm ticket show TNDM-A1B2C3
tndm ticket show TNDM-A1B2C3 --json

# Filter list output by current definition state
tndm ticket list --definition ready

# Compare ticket changes against another branch or worktree
tndm awareness --against branch-a --json

# Keep canonical formatting and fingerprints clean in CI
tndm fmt --check
```

## JSON / API design

The current machine-facing API is the CLI's JSON output.

- `tndm ticket show --json` returns a **schema-versioned ticket envelope** that flattens metadata and state and includes `content_path`.
- `tndm ticket list --json` returns `{ "schema_version": 1, "tickets": [...] }`.
- `tndm ticket task list --json` returns a **bare JSON array of tasks**.
- `tndm awareness --json` returns a schema-versioned change report keyed by `against` and `tickets`.

Example `ticket show --json` shape:

```json
{
  "schema_version": 1,
  "id": "TNDM-A1B2C3",
  "title": "Refresh README overview",
  "type": "chore",
  "priority": "p2",
  "effort": "s",
  "depends_on": [],
  "tags": ["definition:ready", "docs"],
  "documents": [
    { "name": "content", "path": "content.md" },
    { "name": "task-01", "path": "tasks/task-01.md" }
  ],
  "status": "in_progress",
  "updated_at": "2026-05-20T22:02:43.273163Z",
  "revision": 3,
  "document_fingerprints": {
    "content": "sha256:...",
    "task-01": "sha256:..."
  },
  "tasks": [
    {
      "number": 1,
      "title": "Rewrite README capabilities section",
      "status": "todo",
      "files": ["README.md"],
      "verification": "manual review against --help output",
      "detail_path": "tasks/task-01.md"
    }
  ],
  "content_path": ".tndm/tickets/TNDM-A1B2C3/content.md"
}
```

Example awareness diff shape:

```json
{
  "schema_version": 1,
  "against": "HEAD",
  "tickets": [
    {
      "id": "TNDM-A1B2C3",
      "change": "diverged",
      "fields": {
        "status": { "current": "in_progress", "against": "todo" },
        "tags": {
          "current": ["definition:ready", "docs"],
          "against": []
        },
        "documents": [
          { "name": "task-01", "current": "sha256:...", "against": "" }
        ],
        "tasks": {
          "current": [
            {
              "number": 1,
              "title": "Rewrite README capabilities section",
              "status": "todo",
              "files": ["README.md"],
              "verification": "manual review against --help output",
              "detail_path": "tasks/task-01.md"
            }
          ],
          "against": []
        }
      }
    }
  ]
}
```

Awareness diffs currently report field-level changes for:

- status
- priority
- effort
- title
- type
- depends_on
- tags
- document fingerprints
- tasks

## Architecture at a glance

This repo is a Rust workspace with strict dependency boundaries:

- `crates/tandem-core` — domain types, validation, and ports; must remain IO-free
- `crates/tandem-storage` — filesystem-backed ticket storage and fingerprint handling
- `crates/tandem-repo` — git/worktree awareness adapter
- `crates/tandem-cli` — the only CLI crate; produces `tndm`
- `crates/xtask` — developer tooling and architecture checks
- `plugins/supi-flow` — PI extension for spec-driven workflows built on top of TNDM tickets

See [`docs/architecture.md`](docs/architecture.md) for the enforced dependency rules.

## PI extension

[`plugins/supi-flow/`](plugins/supi-flow/) is a **PI extension** that layers a spec-driven
workflow on top of tandem tickets: **brainstorm → plan → apply → archive**.

It ships with:

- 5 custom PI tools: `supi_tndm_cli`, `supi_flow_start`, `supi_flow_plan`, `supi_flow_complete_task`, `supi_flow_close`
- 5 auto-discovered skills
- 1 prompt template

Install from npm:

```sh
pi install npm:@mrclrchtr/supi-flow
```

## Project status

`tandem` is **pre-1.0 and under active development**.

- The CLI, document registry, task workflow, and awareness features work today.
- The on-disk format and JSON surface are intentionally deterministic.
- The user-facing contract is still centered on the CLI; deeper library APIs may continue to evolve before 1.0.

## Documentation

| Doc | What you'll find |
|-----|------------------|
| [`docs/vision.md`](docs/vision.md) | Product goals, workflows, and V1 scope |
| [`docs/architecture.md`](docs/architecture.md) | Workspace structure and enforced dependency rules |
| [`docs/decisions.md`](docs/decisions.md) | Design rationale and storage / awareness decisions |
| [`docs/references.md`](docs/references.md) | Competitive analysis and related projects |
| [`docs/releasing.md`](docs/releasing.md) | Release process and automation |
| [`CHANGELOG.md`](CHANGELOG.md) | Release history |

## Getting help

- **Bug reports / feature requests** — open a [GitHub issue](https://github.com/mrclrchtr/tandem/issues)
- **Questions / discussion** — start a [GitHub discussion](https://github.com/mrclrchtr/tandem/discussions)
- **CLI help** — `tndm --help` for the full command reference

## Contributing

`tandem` is a Rust workspace managed with `mise`.

```sh
mise install        # install toolchains
mise run test       # run the test suite
mise run check      # fmt + compile + arch + clippy
```

See [`docs/architecture.md`](docs/architecture.md) for development conventions and
[`docs/releasing.md`](docs/releasing.md) for the release process.

## License

`tandem` is licensed under [Apache 2.0](LICENSE).
