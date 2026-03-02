# tandem (`tndm`)

Git-aware ticket coordination system for AI agents in a monorepo.

Project vision: `docs/vision.md`.

## Development

- Toolchain manager: `mise`
- Rust version source of truth: `rust-toolchain.toml`
- Build: `cargo build`
- Test: `mise run test`
- Compile: `mise run compile`
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
cargo run --bin tndm -- --help
```
