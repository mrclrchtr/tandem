# Repository Guidelines

## Overview

`tandem` is a git-aware ticket coordination system for AI agents in a monorepo. It stores ticket state in the
repository (designed to work across branches and git worktrees) and exposes a deterministic CLI (`tndm`) for both
humans and agents. The repo-local ticket files are the system of record; no central service is required.

## Project Structure & Module Organization

- `src/main.rs`: CLI entrypoint (`tndm`) built with `clap` derive.
- `src/lib.rs`: Core library (ticket model, git/worktree awareness, deterministic file formatting).
- `docs/`: Product/architecture docs. Start with `docs/vision.md`.
- `target/`: Local build output (do not commit).
- `.agents/`, `.claude/`: Agent tooling/config (kept out of hook file selection).

## Build, Test, and Development Commands

Tooling is managed via `mise` (Rust version comes from `rust-toolchain.toml`).

```sh
mise install                 # install tools
mise run hooks-install       # install git hooks (hk)
cargo build                  # build
cargo run --bin tndm -- --help
mise run fmt                 # rustfmt check (via hk)
mise run fmt-fix             # apply rustfmt
mise run compile             # cargo check --workspace --all-targets --all-features --locked
mise run clippy              # clippy with -D warnings
mise run test                # cargo test --workspace --locked
mise run check               # fmt + compile + clippy + test
```

Hooks can also be run directly:

```sh
hk run check
hk run fix
```

## Coding Style & Naming Conventions

- Formatting: `rustfmt` (run `mise run fmt` / `mise run fmt-fix`).
- Linting: `clippy` with warnings treated as errors; keep the tree warning-free.
- Safety: `unsafe` is forbidden (`[lints.rust] unsafe_code = "forbid"`).
- Naming: modules/functions `snake_case`, types `CamelCase`, constants `SCREAMING_SNAKE_CASE`.

## Testing Guidelines

- Use Rust’s built-in test harness (`#[test]`).
- Prefer unit tests colocated with code (`mod tests { ... }`); add integration tests under `tests/` as needed.
- Keep tests deterministic (no network, stable temp paths).

## Commit Guidelines

- Commit messages follow Conventional Commits: `type(scope): summary`.
- Run tests before committing (`mise run test`).
