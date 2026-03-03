# Architecture

This repository is a Rust workspace designed around strict separation of concerns:

- `tandem-core` is the domain layer (pure logic + invariants).
- Adapter crates provide IO (filesystem, git/worktrees, etc.) behind core-defined traits.
- `tandem-cli` is the only CLI crate (user-facing command is `tndm`).

## Crates

- `crates/tandem-core`
  - Domain types, validation, and interfaces ("ports") that the rest of the system implements.
  - Policy: no filesystem/process spawning/printing side effects.
- `crates/tandem-storage`
  - Filesystem-backed ticket storage and deterministic parsing/formatting (adapter).
- `crates/tandem-repo`
  - Git/worktree awareness and ref-based change detection (adapter).
- `crates/tandem-cli`
  - CLI argument parsing and rendering.
  - Produces the `tndm` binary.
- `crates/xtask`
  - Developer tooling (architecture checks).

## Dependency Direction

Intended dependency graph:

- `tandem-core` has no workspace-crate dependencies.
- `tandem-storage -> tandem-core`
- `tandem-repo -> tandem-core`
- `tandem-cli -> tandem-core + tandem-storage + tandem-repo`
- `xtask` must not depend on workspace crates.

External dependencies policy:

- Only `tandem-cli` may depend on `clap`.

## Enforcement

Architecture boundaries are enforced in two ways:

1. `cargo xtask check-arch` validates crate dependency edges (and the `clap` placement rule) using `cargo metadata`.
2. Clippy configuration disallows common IO/process APIs in `tandem-core` (for example `std::process::Command` and
   selected `std::fs::*` helpers).

Run checks locally:

```sh
mise run arch
```

CI runs the same check as a dedicated "Architecture" step.

## Implementation Notes

- The CLI crate name is `tandem-cli`; the installed command name remains `tndm`.
- Adding or renaming workspace crates requires updating `crates/xtask/src/main.rs` (`WORKSPACE_CRATES` and edge rules),
  and CI will fail until that is done.
- The `docs/vision.md` file describes product goals; this document describes code organization and enforcement.
