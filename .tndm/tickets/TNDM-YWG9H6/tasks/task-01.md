# Task 1: Extract parse_meta helper from load_ticket in tandem-storage

## Goal

Extract the metadata parsing logic from `load_ticket` (lines ~448–540 in `crates/tandem-storage/src/lib.rs`) into a private helper function `parse_meta`.

## What to change

In `crates/tandem-storage/src/lib.rs`:

1. Add a private function **before** `impl TicketStore for FileTicketStore`:

```rust
fn parse_meta(
    raw: RawTicketMeta,
    id: &TicketId,
    meta_path: &Path,
) -> Result<TicketMeta, StorageError> {
    // ... extracted logic
}
```

2. Move these blocks from `load_ticket` into `parse_meta`:
   - depends_on parsing, sorting, dedup
   - tags sorting, dedup
   - `TicketMeta::new` construction
   - ticket_type parsing (if present)
   - priority parsing (if present)
   - effort parsing (if present)
   - depends_on and tags assignment
   - legacy document migration (documents.is_none() → inject default)

3. Replace the extracted code in `load_ticket` with a single call: `let meta = parse_meta(raw_meta, id, &meta_path)?;`

4. Preserve exact error messages and path references in error formatting.

## Verification

- `mise run test` passes all existing tests
- `./tndm-dev ticket show TNDM-YWG9H6` works (load_ticket exercised)
- All error messages unchanged (verified by existing test assertions)
- `mise run clippy` clean

## Notes

- `parse_meta` is a private free function (not a method) — it needs `RawTicketMeta`, `TicketId`, and `Path` as inputs.
- Keep the raw types (`RawTicketMeta`, etc.) above the function; they're already defined in the file.
- The function signature uses `StorageError` for consistency with the existing error type.
