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

Tooling is managed via `mise`; Rust version is pinned in `rust-toolchain.toml`.

```sh
mise install
mise run hooks-install

cargo build
./tndm-dev --help
./tndm-dev ticket list

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
