# Refactor: Extract shared test infrastructure for CLI integration tests

## Problem

The CLI integration test suite (~3,500 lines across 7 files) has massive duplication in setup boilerplate. Every test manually:

1. Creates a tempdir + `.git` marker (2 lines)
2. Runs the CLI binary via `Command::new(env!("CARGO_BIN_EXE_tndm"))` with verbose `.args()`, `.current_dir()`, `.output()`, and assertion boilerplate (6-8 lines per invocation)
3. Extracts stdout as UTF-8 from the raw output (1 line)

The shared `common/mod.rs` has only two tiny helpers (`write_prefix_config`, `create_test_ticket`). Additional helpers (`run_git`, `write_ticket`) are duplicated locally in `awareness_cli_tests.rs`, and a duplicate `create_test_ticket` exists in `ticket_config_tests.rs`.

## Goal

Introduce a `TestRepo` struct in `common/mod.rs` that provides a fluent API for test setup, eliminating boilerplate. Each test shrinks by 30-50% while remaining readable and self-contained.

## Approach

**Introduce a `TestRepo` struct, not a god builder.**

A single struct with methods, not a builder pattern with chainable state. This keeps tests readable — each method call is a clear action — without adding abstraction overhead.

### API design

```rust
// common/mod.rs

pub struct TestRepo {
    root: TempDir,
    binary: PathBuf,  // cached binary path
}

impl TestRepo {
    /// Creates a temp dir with .git marker and locates the tndm binary.
    pub fn new() -> Self;

    /// Creates a temp dir with .git marker AND writes .tndm/config.toml.
    pub fn with_config(prefix: &str) -> Self;

    /// Returns the repo root path.
    pub fn path(&self) -> &Path;

    /// Creates a ticket via the CLI binary. Asserts success.
    /// If id is Some, passes --id; otherwise uses auto-generated ID.
    pub fn create_ticket(&self, id: Option<&str>, title: &str);

    /// Creates a ticket by writing files directly (bypasses CLI).
    /// Useful for testing CLI against pre-existing or malformed state.
    pub fn write_ticket(&self, id: &str, title: &str, status: &str, priority: &str, deps: &[&str]);

    /// Runs the tndm CLI and returns Output (does NOT assert success).
    pub fn run(&self, args: &[&str]) -> std::process::Output;

    /// Runs and asserts success, returns stdout as String.
    pub fn run_assert(&self, args: &[&str]) -> String;

    /// Runs with --json appended, asserts success, returns parsed serde_json::Value.
    pub fn run_json(&self, args: &[&str]) -> serde_json::Value;

    /// Runs git commands in the repo root.
    pub fn run_git(&self, args: &[&str]);

    /// Reads a file from .tndm/tickets/{id}/{filename}.
    pub fn read_ticket_file(&self, id: &str, filename: &str) -> String;
}
```

### Migration strategy

Each test file is migrated independently. The refactoring is mechanical:

| Before | After |
|--------|-------|
| `let repo_root = tempfile::tempdir().expect("tempdir");` | `let repo = TestRepo::new();` |
| `fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");` | *(handled by new())|
| `create_test_ticket(repo_root.path(), "ID", "Title");` | `repo.create_ticket(Some("ID"), "Title");` |
| `Command::new(env!("CARGO_BIN_EXE_tndm")).args([...]).current_dir(repo_root.path()).output().expect("...")` | `repo.run(&[...])` |
| `assert!(output.status.success(), ...);` + `String::from_utf8(output.stdout)` | `repo.run_assert(&[...])` |
| `serde_json::from_str(&stdout)` after `--json` output | `repo.run_json(&[...])` |
| `write_prefix_config(repo_root.path(), "PROJ");` | `TestRepo::with_config("PROJ")` |
| `run_git(repo_root.path(), &[...]);` | `repo.run_git(&[...])` |

### What stays the same

- Test function signatures and `#[test]` attributes
- Test logic and assertions
- Error messages in assertions
- The `common/mod.rs` module declaration

### Non-goals

- No changes to test logic or assertions
- No new abstractions beyond `TestRepo`
- No changes to how the `tndm` binary is built or invoked
- No changes to non-test source code

### Files modified

| File | Change |
|------|--------|
| `crates/tandem-cli/tests/common/mod.rs` | Add `TestRepo` struct with all methods; keep existing helpers |
| `crates/tandem-cli/tests/ticket_create_tests.rs` | Migrate to `TestRepo` |
| `crates/tandem-cli/tests/ticket_list_tests.rs` | Migrate to `TestRepo` |
| `crates/tandem-cli/tests/ticket_update_tests.rs` | Migrate to `TestRepo` |
| `crates/tandem-cli/tests/ticket_task_tests.rs` | Migrate to `TestRepo` |
| `crates/tandem-cli/tests/ticket_config_tests.rs` | Migrate to `TestRepo`, remove local `create_test_ticket` |
| `crates/tandem-cli/tests/fmt_cli_tests.rs` | Migrate to `TestRepo` |
| `crates/tandem-cli/tests/awareness_cli_tests.rs` | Migrate to `TestRepo`, remove local `run_git` and `write_ticket` |

### Verification

After each file migration, run its tests to confirm no behavior changed. After all migrations, run the full test suite (`cargo test --workspace`) to confirm nothing is broken.
