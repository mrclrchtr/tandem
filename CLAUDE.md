# CLAUDE.md

Guidance for Claude Code (claude.ai/code) when working in this repository.

## Repository purpose

`tandem` is a git-aware ticket coordination system for AI agents in a monorepo.

It is designed to work across branches and git worktrees. Ticket state is stored in the repository and exposed via a deterministic
`tndm` CLI.

Start with:
- Product goals / design direction: `docs/vision.md`
- Architecture overview: `docs/architecture.md`

## Workspace invariants (Rust)

Workspace crates:
- `crates/tandem-core` — domain logic + validation + core “ports”. Must remain IO-free.
- `crates/tandem-storage` — filesystem storage adapter implementing core ports.
- `crates/tandem-repo` — git/worktree awareness adapter implementing core ports.
- `crates/tandem-cli` — the only CLI crate (produces the `tndm` binary); the only crate allowed to depend on `clap`.
- `crates/xtask` — dev tooling (architecture boundary check).

Enforced dependency direction (validated by `cargo xtask check-arch`; see `crates/xtask/src/main.rs`):
- `tandem-core` has no workspace-crate dependencies
- `tandem-storage -> tandem-core`
- `tandem-repo -> tandem-core`
- `tandem-cli -> tandem-core + tandem-storage + tandem-repo`
- Only `tandem-cli` may depend on `clap`

Sources of truth (enforced by tooling):
- Architecture boundaries + “clap only in CLI”: `crates/xtask/src/main.rs`
  (invoked via `cargo xtask check-arch`; alias in `.cargo/config.toml`)
- `tandem-core` IO bans: `clippy.toml`
- No `unsafe`: workspace lints in root `Cargo.toml`

If you add/rename workspace crates, update `crates/xtask/src/main.rs` (workspace crate list + edge rules).

Design direction lives in `docs/vision.md` (avoid encoding future plans here).

## Common development commands

Tooling is managed via `mise`; Rust version is pinned in `rust-toolchain.toml`.

```sh
mise install                 # install tools
mise run hooks-install       # install git hooks (hk)

mise run fmt                 # rustfmt check
mise run fmt-fix             # apply rustfmt
mise run compile             # cargo check --workspace --all-targets --all-features --locked
mise run arch                # cargo xtask check-arch
mise run clippy              # clippy with -D warnings
mise run test                # cargo test --workspace --locked
mise run check               # fmt + compile + arch + clippy + test
mise run fix                 # auto-fixes (currently formatting)
```

## CI notes

GitHub Actions runs the same `mise` tasks (`fmt`, `compile`, `arch`, `clippy`, `test`) (see `.github/workflows/ci.yml`).
Compile/clippy/test use `--locked`, so keep `Cargo.lock` in sync.
