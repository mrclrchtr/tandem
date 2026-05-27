# Task 1: Create TestRepo struct in common/mod.rs

## Goal

Add a `TestRepo` struct to `crates/tandem-cli/tests/common/mod.rs` that consolidates all shared test infrastructure: tempdir creation, CLI binary invocation, ticket creation (both CLI-based and filesystem-based), and git command helpers.

## Files

**`crates/tandem-cli/tests/common/mod.rs`** — add `TestRepo` struct and all methods.

### TestRepo API

```rust
pub struct TestRepo {
    root: TempDir,
}

impl TestRepo {
    /// Creates a temp dir with .git marker.
    pub fn new() -> Self { ... }

    /// Creates a temp dir with .git marker and writes .tndm/config.toml with the given prefix.
    pub fn with_config(prefix: &str) -> Self { ... }

    /// Returns the repo root path.
    pub fn path(&self) -> &Path { ... }

    /// Creates a ticket via the CLI binary. Asserts success.
    pub fn create_ticket(&self, id: Option<&str>, title: &str) { ... }

    /// Creates a ticket by writing meta.toml/state.toml/content.md directly.
    pub fn write_ticket(&self, id: &str, title: &str, status: &str, priority: &str, deps: &[&str]) { ... }

    /// Runs the tndm CLI and returns the raw Output.
    pub fn run(&self, args: &[&str]) -> std::process::Output { ... }

    /// Runs the tndm CLI, asserts success, returns stdout as String.
    pub fn run_assert(&self, args: &[&str]) -> String { ... }

    /// Runs with --json appended, asserts success, returns parsed Value.
    pub fn run_json(&self, args: &[&str]) -> serde_json::Value { ... }

    /// Runs a git command in the repo root.
    pub fn run_git(&self, args: &[&str]) { ... }

    /// Reads a file from .tndm/tickets/{id}/{filename}.
    pub fn read_ticket_file(&self, id: &str, filename: &str) -> String { ... }
}
```

### Implementation notes

- `new()` uses `tempfile::tempdir()` and creates `.git` dir inside it
- `with_config(prefix)` calls `new()` then writes `.tndm/config.toml` using the existing `write_prefix_config` helper
- `run()` uses `Command::new(env!("CARGO_BIN_EXE_tndm"))` with `.current_dir(self.path())`
- `run_assert()` calls `run()`, asserts `output.status.success()`, returns `String::from_utf8(output.stdout).expect(...)`
- `run_json()` calls `run_assert()` with `--json` appended, then `serde_json::from_str()`
- `run_git()` uses `Command::new("git")` with `-C self.path()` prefix and asserts success
- `write_ticket()` mirrors the existing implementation from `awareness_cli_tests.rs` (creates dirs, writes meta.toml/state.toml/content.md)
- `create_ticket()` wraps the existing `create_test_ticket` helper, taking `Option<&str>` for `id` (None = auto-generated)
- Keep the existing `write_prefix_config` and `create_test_ticket` helpers (they may still be used, or the latter can delegate to `TestRepo`)

### Verification

```bash
cargo test -p tandem-cli --test common 2>&1 || echo "No test binary for common module — compile check only"
cargo check -p tandem-cli 2>&1
```
