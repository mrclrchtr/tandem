# Task 8: Migrate awareness_cli_tests.rs to TestRepo

## Goal

Migrate `crates/tandem-cli/tests/awareness_cli_tests.rs` (332 lines) to use `TestRepo`. Remove the local `run_git` and `write_ticket` helper functions.

## Files

**`crates/tandem-cli/tests/awareness_cli_tests.rs`** — mechanical migration + cleanup.

### Special: Remove local helpers

This file defines two local helpers at the bottom:
- `fn run_git(repo_root: &Path, args: &[&str])` (line 278)
- `fn write_ticket(repo_root: &Path, id, title, status, priority, depends_on)` (line 300)

After migration, these are no longer called. Remove them. Callers switch to `repo.run_git(...)` and `repo.write_ticket(...)`.

### Pattern replacements

- Setup replaces the 4-line git init pattern:
  ```rust
  // Before
  let repo_root = tempfile::tempdir().expect("tempdir");
  run_git(repo_root.path(), &["init", "-b", "main"]);
  run_git(repo_root.path(), &["config", "user.name", "Test User"]);
  run_git(repo_root.path(), &["config", "user.email", "test@example.com"]);

  // After
  let repo = TestRepo::new();
  repo.run_git(&["init", "-b", "main"]);
  repo.run_git(&["config", "user.name", "Test User"]);
  repo.run_git(&["config", "user.email", "test@example.com"]);
  ```

- `write_ticket(repo_root.path(), ...)` → `repo.write_ticket(...)`
- `Command::new(env!("CARGO_BIN_EXE_tndm"))...` → `repo.run(...)` / `repo.run_assert(...)`
- `repo_root.path().join(...)` → `repo.path().join(...)`
- `fs::remove_dir_all(repo_root.path().join(...))` → `fs::remove_dir_all(repo.path().join(...))`

### Tests

1. `awareness_prints_empty_json_when_snapshots_match`
2. `awareness_reports_added_current_added_against_and_diverged_sorted`
3. `awareness_errors_for_invalid_ref`
4. `awareness_text_output_shows_human_readable_format`
5. `awareness_text_output_empty_shows_no_changes`

### Note

`TestRepo::new()` creates `.git` as a directory, but awareness tests need a git repo (with `git init`). The `.git` dir created by `new()` is just a marker; `git init` will convert it. Alternative: add a `init_git()` method to `TestRepo` that runs `git init -b main` and configures user. But to keep the first task simple, awareness tests can call `repo.run_git()` manually for git init as shown above.

### Verification

```bash
cargo test -p tandem-cli awareness -- --nocapture
```
