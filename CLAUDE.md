# CLAUDE.md

Guidance for Claude Code (claude.ai/code) when working in this repository.

## Repository purpose

`tandem` is a git-aware ticket coordination system for AI agents in a monorepo.

It is designed to work across branches and git worktrees. Ticket state is stored in the repository and exposed via a deterministic `tndm` CLI. Repo-local ticket files are the system of record; no central service is required.

Start with:
- Product vision: `docs/vision.md`
- Design decisions: `docs/decisions.md`
- Architecture overview: `docs/architecture.md`

## Project structure

- `crates/tandem-core` — domain logic + validation + core ports; must remain IO-free.
- `crates/tandem-storage` — filesystem storage adapter implementing core ports.
- `crates/tandem-repo` — git/worktree awareness adapter implementing core ports.
- `crates/tandem-cli` — CLI crate producing `tndm`; the only crate allowed to depend on `clap`.
- `crates/xtask` — dev tooling, including `cargo xtask check-arch`.
- `docs/` — product and architecture docs; start with `docs/vision.md`, `docs/decisions.md`, and `docs/architecture.md`.
- `target/` — local build output; do not commit.
- `plugins/tndm` — Claude Code and Codex plugin: skills, hooks, and slash commands that teach agents to use the `tndm` CLI.
- `plugins/supi-flow` — PI-only extension: spec-driven workflow (brainstorm → plan → apply → archive) coupled to TNDM ticket coordination. Registers custom tools (`supi_tndm_cli`, `supi_flow_*`) and auto-discovers 6 flow skills. Not a Claude Code plugin.
- Before using any `tndm` command, read `plugins/tndm/skills/ticket/references/command-reference.md` for available subcommands and flags.
- When changing CLI behavior, update the plugin command reference: `plugins/tndm/skills/ticket/references/command-reference.md`.
- When changing plugin behavior, bump `version` in `plugins/tndm/.claude-plugin/plugin.json` and keep `plugins/tndm/.codex-plugin/plugin.json` in sync.
- Prompt-based hooks (Stop, SubagentStop, etc.) must respond with `{"ok": true/false, "reason": "..."}`. Use `$ARGUMENTS` in the prompt to inject hook input.
- `.agents/`, `.claude/` — agent tooling/config kept out of hook file selection.

## Workspace invariants (Rust)

Enforced dependency direction (validated by `cargo xtask check-arch`; see `crates/xtask/src/main.rs`):
- `tandem-core` has no workspace-crate dependencies.
- `tandem-storage -> tandem-core`
- `tandem-repo -> tandem-core`
- `tandem-cli -> tandem-core + tandem-storage + tandem-repo`
- Only `tandem-cli` may depend on `clap`.

Sources of truth (enforced by tooling):
- Architecture boundaries and “clap only in CLI”: `crates/xtask/src/main.rs`
  (invoked via `cargo xtask check-arch`; alias in `.cargo/config.toml`)
- `tandem-core` IO bans: `clippy.toml`
- No `unsafe`: workspace lints in root `Cargo.toml`

If you add or rename workspace crates, update `crates/xtask/src/main.rs` to keep the workspace crate list and edge rules current.

Product vision lives in `docs/vision.md`; design decisions in `docs/decisions.md`. Avoid encoding future plans here.

## Common development commands

Tooling is managed via `mise` (except Rust itself — Rust is pinned by `rust-toolchain.toml` and managed via `rustup`, not mise).

```sh
mise install
mise run hooks-install

cargo build
./tndm-dev --help
./tndm-dev ticket list
./tndm-dev fmt --check  # verify canonical .tndm formatting after serializer/CLI format changes

mise run fmt
mise run fmt-fix
mise run compile
mise run arch
mise run clippy
mise run test
mise run check
mise run fix

hk run check
hk run fix
```

## Git hooks and `hk`

- `cargo-clippy` runs in hk `pre-commit`, `pre-push`, and `check`.
- `cargo-test` is intentionally not in `hk.pkl`; use `mise run test` (runs `cargo test --workspace --locked`).
- Renovate updates `hk.pkl`. If hk-related checks fail after a version bump, update `hk` in `mise.toml` and run `mise install` to refresh `mise.lock`.

## Verification shortcuts

- After changing ticket serialization, formatting, or canonical TOML output, run `./tndm-dev fmt --check`.

## Coding and testing conventions

- `rustfmt` is the formatter; `clippy` runs with warnings treated as errors.
- `unsafe` is forbidden (`[lints.rust] unsafe_code = "forbid"`).
- Use Rust’s built-in test harness (`#[test]`).
- Prefer unit tests colocated with the code (`mod tests { ... }`); add integration tests under `tests/` when needed.
- Keep tests deterministic: no network access and stable temp paths.
- Naming: modules/functions `snake_case`, types `CamelCase`, constants `SCREAMING_SNAKE_CASE`.

## CI notes

GitHub Actions runs the same `mise` tasks (`fmt`, `compile`, `arch`, `clippy`, `test`) in `.github/workflows/ci.yml`. Compile, clippy, and test use `--locked`, so keep `Cargo.lock` in sync. CI also verifies `mise.lock`, so refresh it when changing tool versions.

## Commit guidelines

- Commit messages follow Conventional Commits: `type(scope): summary`.
- Run `mise run test` before committing.

## Adding a new optional field to TicketMeta

Touch all five sites in order:
1. `crates/tandem-core/src/ticket.rs` — add field to struct, `new()`, and `to_canonical_toml()` (hand-written; not auto-serialized). Follow `TicketPriority` as the canonical enum pattern.
2. `crates/tandem-core/src/awareness.rs` — add field to `AwarenessFieldDiffs`, compute diff in `between()`, add to `is_empty()`.
3. `crates/tandem-storage/src/lib.rs` — add `Option<String>` to `RawTicketMeta`, parse after loading.
4. `crates/tandem-cli/src/main.rs` — add clap flag; in `handle_ticket_update`, add `&& field.is_none()` at **both** sites of the `no_explicit_update` guard.
5. `plugins/tndm/skills/ticket/references/command-reference.md` — document flag, update defaults line, add enum table, bump `plugin.json` version in both `.claude-plugin/` and `.codex-plugin/`.
