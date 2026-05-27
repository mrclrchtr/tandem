# Task 2: Extract parse_state helper from load_ticket in tandem-storage

## Goal

Extract the state parsing logic from `load_ticket` (lines ~542–551 in `crates/tandem-storage/src/lib.rs`) into a private helper function `parse_state`.

## What to change

In `crates/tandem-storage/src/lib.rs`:

1. Add a private function right after `parse_meta`:

```rust
fn parse_state(
    raw: RawTicketState,
    state_path: &Path,
) -> Result<TicketState, StorageError> {
    // ... extracted logic
}
```

2. Move these blocks from `load_ticket` into `parse_state`:
   - `TicketState::new(raw_state.updated_at, raw_state.revision)` construction
   - status parsing from `raw_state.status`
   - document_fingerprints assignment
   - tasks parsing: `raw_state.tasks.unwrap_or_default()` → `Vec<Task>` mapping (RawTaskStatus → TaskStatus)

3. Replace the extracted code in `load_ticket` with a single call: `let state = parse_state(raw_state, &state_path)?;`

## Verification

- `mise run test` passes all existing tests
- `./tndm-dev ticket show TNDM-YWG9H6` works (load_ticket exercised)
- All error messages unchanged

## Notes

- This is the simpler of the two extractions — state parsing is only ~10 lines.
- The `RawTaskStatus` enum mapping stays inside `parse_state`.
