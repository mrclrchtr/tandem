# Version Sync Design

Date: 2026-03-30

## Problem

Three files hold version numbers that should stay in sync, but diverge today:

| File | Current version |
|---|---|
| `Cargo.toml` (workspace) | `0.1.0` |
| `plugin/tndm/.claude-plugin/plugin.json` | `0.3.3` |
| `.claude-plugin/marketplace.json` | `0.1.1` |

There is no automation to keep them aligned.

## Design

**Source of truth:** `workspace.package.version` in root `Cargo.toml`.

**Derived files:** `plugin/tndm/.claude-plugin/plugin.json` (field `version`), `.claude-plugin/marketplace.json` (field `version`).

### `cargo xtask sync-version`

New xtask subcommand with two modes:

- **Default (write mode):** Reads `workspace.package.version` from root `Cargo.toml` using the `toml` crate. Writes the version to both JSON files using `serde_json`, preserving formatting. Prints a summary of changes. Exits 0.
- **`--check` (dry-run mode):** Reads and compares without writing. Exits 0 if all files match, exits 1 with a message listing mismatches. Intended for CI.

**Dependencies:** Add `toml` and `serde_json` (both already workspace deps) to `xtask/Cargo.toml`.

### `mise run sync-version`

New mise task: `cargo xtask sync-version`.

### `mise run bump <version>`

New mise task that:

1. Updates `workspace.package.version` in root `Cargo.toml` (using `sed` or `toml`-aware editing).
2. Runs `cargo xtask sync-version` to propagate.

### CI integration

Add `cargo xtask sync-version --check` step to `.github/workflows/ci.yml`, after the existing `cargo xtask check-arch` step.

### `mise run check` update

Add `cargo xtask sync-version --check` to the existing `check` task so local `mise run check` also catches drift.

## Out of scope

- Automatic version bumping based on conventional commits.
- Changelog generation.
- Publishing or release automation.
