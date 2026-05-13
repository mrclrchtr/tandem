## Context
`create_ticket` and `update_ticket` both implement the same atomic-write dance: clean stale temp dir → create temp → write files → `fs::rename` → cleanup on error. The rollback logic in `update_ticket` is slightly more complex but structurally identical.

## Suggestion
Extract a helper like:
```rust
fn atomic_write_dir(
    final_path: &Path,
    write_fn: impl FnOnce(&Path) -> Result<(), StorageError>,
) -> Result<(), StorageError>;
```

## First Task Before Planning
- [ ] **Validate this refactoring for correctness, best practices, and actual value to the project.** Ensure the helper handles the `update_ticket` two-phase rename (existing → old, staging → final) correctly. Verify error cleanup and rollback semantics are preserved. Consider whether the closure-based API is ergonomic or if a builder/struct approach is better.
