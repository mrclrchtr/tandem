# Task 6: Verify: full test suite, architecture checks, and canonical TOML

## Goal

Confirm all three refactorings work together end-to-end with no regressions.

## What to do

Run the full verification suite from the repo root:

```sh
mise run check    # fmt + compile + arch + clippy
mise run test     # cargo test --workspace
./tndm-dev fmt --check  # canonical TOML verification
```

## Acceptance criteria

- `mise run check` exits 0 (formatting clean, compilation succeeds, architecture boundaries intact, no clippy warnings)
- `mise run test` exits 0 (all workspace tests pass)
- `./tndm-dev fmt --check` exits 0 (canonical TOML output unchanged)
- `git diff --stat` shows only expected files: `crates/tandem-storage/src/lib.rs`, `crates/tandem-core/src/ticket/mod.rs`, `crates/tandem-cli/src/cli/ticket.rs`

## Notes

This is the integration gate. Each task already has its own unit-level verification; this confirms the assembled change is correct.
