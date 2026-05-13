# Plan: Extract atomic write pattern in tandem-storage

## Summary

The `create_ticket` and `update_ticket` methods in `FileTicketStore` both implement the same atomic-write dance: clean stale temp dir → create temp dir → write files → `fs::rename` → cleanup on error. Extract a private `atomic_write_dir` helper to DRY this pattern. In `update_ticket`, also simplify the two-phase rename (existing → old.tmp, staging → final) to a single atomic rename since the two-phase was over-engineering — `fs::rename` atomically replaces the destination on the same filesystem.

## Files to modify

| File | Change |
|------|--------|
| `crates/tandem-storage/src/lib.rs` | Add `atomic_write_dir` helper; refactor `create_ticket` and `update_ticket` to use it |
| `crates/tandem-storage/tests/ticket_store_tests.rs` | Update `update_ticket_cleans_up_stale_staging_dirs` for new staging dir naming |

---

## Tasks

- [x] **Task 1**: Add the `atomic_write_dir` helper function
  - File: `crates/tandem-storage/src/lib.rs`
  - Add a private generic function after the `FileTicketStore` impl block:
    ```rust
    fn atomic_write_dir<F>(
        final_path: &Path,
        write_fn: F,
    ) -> Result<(), StorageError>
    where
        F: FnOnce(&Path) -> Result<(), StorageError>,
    ```
  - Behavior:
    1. Derive staging path as `final_path.parent()/.{final_name}.tmp`
    2. Clean stale staging dir (`remove_dir_all` if exists)
    3. Create staging dir
    4. Execute `write_fn(&staging)`  — closure writes content into staging
    5. If write_fn returned error: `remove_dir_all(&staging)`, propagate error
    6. `fs::rename(&staging, final_path)` — atomic swap
    7. If rename fails: `remove_dir_all(&staging)`, return error
  - Verification: `cargo build -p tandem-storage`

- [x] **Task 2**: Refactor `create_ticket` to use `atomic_write_dir`
  - File: `crates/tandem-storage/src/lib.rs`
  - Replace lines ~348–398 (stale cleanup → create staging → write files → cleanup) with:
    ```rust
    atomic_write_dir(&ticket_path, |staging| {
        // write meta.toml, state.toml, content.md
        Ok(())
    })?;
    ```
  - Keep `create_dir_all(&tickets_path)` outside the helper (it's setup, not atomic write)
  - Keep fingerprint computation and return-value construction unchanged
  - Verification: `cargo test -p tandem-storage --test ticket_store_tests --test create_ticket_reliability_tests --test awareness_storage_tests`

- [x] **Task 3**: Refactor `update_ticket` to use `atomic_write_dir`
  - File: `crates/tandem-storage/src/lib.rs`
  - Remove `old_path` entirely (no two-phase rename needed)
  - Remove `.id.update.tmp` specific staging — use `atomic_write_dir(&ticket_path, ...)` which derives staging as `.id.tmp`
  - Extract the write logic (meta, state, content, plus extra document copy) into the `write_fn` closure
  - Keep the `ticket_path.is_dir()` existence check before the call
  - Verification: `cargo test -p tandem-storage --test ticket_store_tests`

- [x] **Task 4**: Update `update_ticket_cleans_up_stale_staging_dirs` test
  - File: `crates/tandem-storage/tests/ticket_store_tests.rs`
  - Current test creates `.TNDM-STALE.update.tmp` and `.TNDM-STALE.old.tmp` stale dirs
  - The new code only uses one staging dir `.TNDM-STALE.tmp` (derived by `atomic_write_dir`)
  - Change to create a stale `.TNDM-STALE.tmp` dir and verify it gets cleaned up
  - Verification: `cargo test -p tandem-storage`

## Self-review

1. **Coverage**: All three important behaviors are covered — Task 2 (create), Task 3 (update), Task 4 (test fix).
2. **Placeholder scan**: No TODOs or vague instructions.
3. **Consistency**: The helper uses the same naming scheme (`.final_name.tmp`) that create_ticket already uses; `update_ticket` converges to the same scheme. Error messages and cleanup semantics are preserved.
4. **Right-sized detail**: Each task has exact file paths, the change to make, and a verification command.
