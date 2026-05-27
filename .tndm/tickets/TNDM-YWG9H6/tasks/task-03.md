# Task 3: Deduplicate path computation in handle_task_detail_ensure

## Goal

Eliminate the redundant path computation in `handle_task_detail_ensure` that duplicates logic already in `ensure_canonical_task_detail_doc`.

## What to change

In `crates/tandem-cli/src/cli/ticket.rs`, in `handle_task_detail_ensure` (lines ~873–923):

**Current code** (after the `ensure_canonical_task_detail_doc` call):
```rust
let (_rel_path, created_file) = ensure_canonical_task_detail_doc(...)?;

recompute_ticket_document_fingerprints(...)?;

if let Err(error) = ctx.store.update_ticket(&ticket)... {
    if created_file {
        let _ = fs::remove_file(&abs_path);
    }
    return Err(error);
}

if json {
    println!("{}", serde_json::json!({
        "ticket_id": ticket_id.as_str(),
        "task_number": number,
        "name": doc_name,
        "detail_path": rel_path,
        "path": abs_path.to_string_lossy(),
    }));
}
```

The variables `doc_name`, `rel_path`, and `abs_path` are computed BEFORE the `ensure_canonical_task_detail_doc` call via redundant `canonical_task_detail_doc(number)` and `ticket_dir()` calls.

**Change**:
1. Capture the returned `rel_path` from `ensure_canonical_task_detail_doc` (don't discard with `_`)
2. Compute `abs_path` from the returned `rel_path`
3. Compute `doc_name` from the returned `rel_path` (extract filename without extension: `task-XX` from `tasks/task-XX.md`) — OR keep the `canonical_task_detail_doc(number)` call for doc_name since it's a pure format computation, not duplication

**Simplest correct fix**: Use the returned `rel_path` in the output and for rollback. The `doc_name` can remain from `canonical_task_detail_doc(number)` since `canonical_task_detail_doc` is a trivial format function with no allocation concern.

```rust
let (rel_path, created_file) = ensure_canonical_task_detail_doc(...)?; // no underscore
let (doc_name, _) = canonical_task_detail_doc(number);
let abs_path = ticket_dir(&ctx.repo_root, &ticket_id).join(&rel_path);
```

This eliminates the redundant `canonical_task_detail_doc(number).1` computation that was discarded before.

## Verification

- `mise run test` passes
- Specifically: `cargo test --package tandem-cli handle_task_detail` or equivalent task detail tests
- Manual: `./tndm-dev ticket task detail ensure TNDM-YWG9H6 1` produces correct output

## Notes

- This is the smallest change (~3 lines modified in the `let` bindings)
- The `ensure_canonical_task_detail_doc` function already validates and returns the canonical `rel_path` — the handler was just discarding it
