# CLAUDE.md

Guidance for coding agents working in this repository.

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
- `plugins/supi-flow` — PI-only extension using conventional directory layout (`extensions/`, `skills/`, `prompts/`). Spec-driven workflow (brainstorm → plan → apply → archive) coupled to TNDM ticket coordination. Registers 5 custom tools (`supi_tndm_cli`, `supi_flow_*`) and auto-discovers 5 flow skills + 1 prompt template. Not a Claude Code plugin.
  See `plugins/supi-flow/CLAUDE.md` for detailed guidance.

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

Install via Homebrew (production release):

```sh
brew install mrclrchtr/tap/tandem-cli
```

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

## supi-flow plugin

The `plugins/supi-flow/` directory contains a PI-only extension (not a Claude Code plugin) that implements a spec-driven workflow (brainstorm → plan → apply → archive) coupled to TNDM ticket coordination. See `plugins/supi-flow/CLAUDE.md` for full guidance.

The plugin tools wrap the `tndm` CLI directly — update the CLI help text when changing behavior.

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
- `#[serde(flatten)]` on two structs sharing a field name (e.g., `TicketMeta` + `TicketState` both flattened in `TicketJsonEntry`) causes duplicate-key errors. Use `#[serde(skip)]` or extract a shared parent field.
- `string_enum!` macro in `crates/tandem-core/src/ticket.rs` — use for new string-backed enums; generates `parse()`, `as_str()`, `FromStr`, `Display`, `Serialize` from variant→str mapping (e.g., `InProgress => "in_progress"`)
- `string_enum!` variants: the variant name's `snake_case` must match its `$str` literal — `Serialize` (derive + `rename_all`) and `Display` (`as_str()`) output will diverge silently otherwise. Tests in `macro_generated_impls` catch mismatches.
- Naming: modules/functions `snake_case`, types `CamelCase`, constants `SCREAMING_SNAKE_CASE`.

## CI notes

GitHub Actions runs the same `mise` tasks (`fmt`, `compile`, `arch`, `clippy`, `test`) in `.github/workflows/ci.yml`. Compile, clippy, and test use `--locked`, so keep `Cargo.lock` in sync. CI also verifies `mise.lock`, so refresh it when changing tool versions.

## Commit guidelines

- Commit messages follow Conventional Commits: `type(scope): summary`.
- Run `mise run test` before committing.

## Adding a new optional field to TicketMeta

Touch all five sites in order:
1. `crates/tandem-core/src/ticket.rs` — add field to struct with appropriate serde attributes (`rename`, `skip`, etc.), update `new()`. `to_canonical_toml()` auto-serializes via `toml::to_string()`.
2. `crates/tandem-core/src/awareness.rs` — add field to `AwarenessFieldDiffs`, compute diff in `between()`, add to `is_empty()`.
3. `crates/tandem-storage/src/lib.rs` — add `Option<String>` to `RawTicketMeta`, parse after loading.
4. `crates/tandem-cli/src/cli/ticket.rs` — add clap flag to `TicketCommand::Create` and `TicketCommand::Update` args; add `&& field.is_none()` to both `no_explicit_create` and `no_explicit_update` boolean computations in `handle_ticket_create` and `handle_ticket_update`.
5. `crates/tandem-cli/src/cli/mod.rs` — ensure the new flag is wired into ticket commands with current help text.
