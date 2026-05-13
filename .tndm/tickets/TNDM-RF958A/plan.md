# Plan: Clarify document diff semantics in awareness

## Design summary (from content.md)

The `documents` field in `AwarenessFieldDiffs` currently uses `AwarenessVecDiff` listing **all** registered document names on both sides, not just the ones whose fingerprints changed. This is not useful for consumers.

**Decision:** Replace with a dedicated `AwarenessDocEntry` struct that reports only changed document entries, including the document name and its fingerprint in both `current` and `against` snapshots.

**Validation completed:**
- No known consumer parses the `documents` field with typed shape expectations (pi tool passes JSON verbatim; text formatter skips documents entirely)
- Shape change is safe in practice
- New shape is more informative: consumers can see what the fingerprint changed from/to

## File map

| File | Action | Responsibility |
|------|--------|----------------|
| `crates/tandem-core/src/awareness.rs` | Modify | Add `AwarenessDocEntry` struct; replace `documents` field type; update diff logic; add/update tests |
| `crates/tandem-cli/src/cli/awareness.rs` | Modify | Render document diffs in text output |
| `crates/tandem-core/src/awareness.rs` (tests) | Modify | Update existing document diff test; add new tests for partial changes, added/removed docs |

## Tasks

- [x] **Task 1:** Define `AwarenessDocEntry` struct and update `AwarenessFieldDiffs`
  - File: `crates/tandem-core/src/awareness.rs`
  - Verification: `cargo check -p tandem-core`

- [x] **Task 2:** Update document diff computation in `AwarenessFieldDiffs::between`
  - File: `crates/tandem-core/src/awareness.rs`
  - Verification: `cargo test -p tandem-core awareness::tests`

- [x] **Task 3:** Update and expand tests
  - File: `crates/tandem-core/src/awareness.rs` (tests)
  - Verification: `cargo test -p tandem-core awareness::tests -v`

- [x] **Task 4:** Render documents in text output
  - File: `crates/tandem-cli/src/cli/awareness.rs`
  - Verification: `cargo test -p tandem-cli -v`

- [x] **Task 5:** Build and full test suite
  - Verification: `cargo test --workspace && cargo xtask check-arch`
