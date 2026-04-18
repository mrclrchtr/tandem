# tandem (`tndm`)

Git-aware ticket coordination system for AI agents in a monorepo.

Project vision: `docs/vision.md`.
Design decisions: `docs/decisions.md`.
Architecture overview: `docs/architecture.md`.

## Install

```sh
cargo install --path crates/tandem-cli
```

This places the `tndm` binary into `~/.cargo/bin/`.

## Development

- Toolchain manager: `mise`
- Rust version source of truth: `rust-toolchain.toml`
- Build: `cargo build`
- Test: `mise run test`
- Compile: `mise run compile`
- Architecture: `mise run arch`
- Lint: `mise run clippy`
- Format: `mise run fmt`
- Full check: `mise run check`
- Auto-fix: `mise run fix`

Install tools:

```sh
mise install
```

Install git hooks:

```sh
mise run hooks-install
```

Run hooks manually:

```sh
hk run check
hk run fix
hk run check --all --step cargo-clippy
```

Run `mise` tasks manually:

```sh
mise run check
mise run fmt
mise run compile
mise run arch
mise run clippy
mise run test
mise run fix
```

Notes:

- CI runs `mise` tasks (`fmt`, `compile`, `clippy`, `test`) and uses `--locked` in compile/clippy, so keep `Cargo.lock` up to date.
- Install hooks with `mise run hooks-install` so git hooks always run in a `mise`-managed tool environment.
- `cargo-clippy` runs in hk `pre-commit`, `pre-push`, and `check`.
- `cargo-test` is intentionally not in `hk.pkl`; `mise run test` executes `cargo test --workspace --locked`.
- Renovate updates `hk.pkl`; if hk-related checks fail after a version bump, update `hk` in `mise.toml` and run `mise install` to refresh `mise.lock`.

Run the CLI:

```sh
./tndm-dev --help
./tndm-dev ticket list
./tndm-dev ticket list --definition ready
```

Equivalent direct Cargo invocation:

```sh
cargo run -p tandem-cli --bin tndm -- --help
```

## Agent Plugin

This repo includes agent packaging for both Claude Code and Codex.

Claude Code:

```sh
claude --plugin-dir ./plugins/tndm
```

The repository also exposes a top-level `skills/` symlink that points at `plugins/tndm/skills`, so `npx skills add https://github.com/mrclrchtr/tandem` can discover the same skills from the repo root.

Codex:

- Personal plugin directory: `~/.codex/plugins/tndm`
- Personal marketplace: `~/.agents/plugins/marketplace.json`
- Install `tndm` from that personal marketplace after restarting Codex

See `plugins/tndm/README.md` for current install details and limitations.

Ticket-definition convention:

- Use `definition:questions` when a ticket still has unresolved `Open Questions`.
- Use `definition:ready` when a ticket is currently implementable.
- Keep detailed rationale in `content.md`; use tags as the coarse machine-readable signal.
