# Task 2: Migrate ticket_create_tests.rs to TestRepo

## Goal

Migrate `crates/tandem-cli/tests/ticket_create_tests.rs` (325 lines) to use `TestRepo`, removing all boilerplate.

## Files

**`crates/tandem-cli/tests/ticket_create_tests.rs`** — mechanical migration.

### Pattern replacements

For every test in this file:
- Replace `let repo_root = tempfile::tempdir().expect("tempdir"); fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");` with `let repo = TestRepo::new();`
- Replace `Command::new(env!("CARGO_BIN_EXE_tndm")).args([...]).current_dir(repo_root.path()).output().expect("...")` with `repo.run(&[...])`
- Replace `assert!(output.status.success(), ...); let stdout = String::from_utf8(output.stdout).expect("...");` with `let stdout = repo.run_assert(&[...]);`
- Replace `repo_root.path().join(...)` with `repo.path().join(...)`
- Remove unused imports (`use std::process::Command` if no longer needed; `use std::fs` if still needed for `fs::read_to_string`)

### Tests in this file

1. `ticket_create_prints_generated_id_and_writes_ticket_files` — uses generated ID (no --id), reads files from ticket dir
2. `ticket_create_json_outputs_full_ticket_envelope` — uses --json
3. `ticket_create_uses_definition_friendly_default_template` — reads content.md
4. `ticket_create_with_all_metadata_flags` — creates prereq tickets, uses many flags
5. `ticket_create_with_priority_flag` — basic create + show
6. `ticket_create_rejects_invalid_priority` — expects failure
7. `ticket_create_rejects_invalid_depends_on` — expects failure
8. `ticket_create_with_effort_flag` — effort flag

### Special cases

- `ticket_create_with_all_metadata_flags`: The for-loop creating prereq tickets uses `.status.success().then_some(()).expect(...)`. This can become `repo.create_ticket(Some(id), "prereq");`.
- Tests that expect failure (invalid priority, invalid depends_on) should use `repo.run(...)` (no assert) and check `!output.status.success()`.

### Verification

```bash
cargo test -p tandem-cli ticket_create -- --nocapture
```
