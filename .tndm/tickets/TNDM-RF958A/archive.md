# Archive

## Verification Results

### Task 1: Define AwarenessDocEntry struct and update AwarenessFieldDiffs
- Added `AwarenessDocEntry` struct with `{name, current, against}` fields
- Changed `documents` field type from `Option<AwarenessVecDiff>` to `Option<Vec<AwarenessDocEntry>>`
- Verification: `cargo check -p tandem-core` — passed

### Task 2: Update document diff computation in AwarenessFieldDiffs::between
- Replaced diff logic: now builds `Vec<AwarenessDocEntry>` containing only changed documents
- Uses `BTreeSet` for stable ordering
- Added docs use `""` for fingerprints missing from one side
- Verification: `cargo test -p tandem-core awareness::tests` — 19 passed

### Task 3: Update and expand tests
- Updated existing test to check new shape (name, current, against)
- Added 4 new tests: partial changes, added doc, removed doc, unchanged
- Verification: `cargo test -p tandem-core awareness::tests -v` — 19 passed

### Task 4: Render documents in text output
- Added rendering block in `format_awareness_text()`
- Format: `  documents:\n    <name>: <current_fp> -> <against_fp>`
- Previously documents were invisible in text output
- Verification: `cargo test -p tandem-cli -v` — 51 passed

### Task 5: Build and full test suite
- `cargo test --workspace` — 132 passed, 1 ignored
- `cargo xtask check-arch` — passed
- `cargo clippy --all-targets -- -D warnings` — clean

### Documentation updated
- `docs/decisions.md` — added `effort` and `documents` to field diffs list, documented AwarenessDocEntry format

### Key design
- New JSON shape: `"documents": [{"name": "content", "current": "sha256:abc", "against": "sha256:def"}]`
- Only changed documents are included
- No known consumer breaks (pi tool passes JSON verbatim; text formatter now renders them)
