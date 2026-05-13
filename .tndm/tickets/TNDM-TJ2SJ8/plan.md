## Plan: Replace hand-rolled TOML with toml+serde in tandem-core

### Files to modify
- `crates/tandem-core/Cargo.toml` — add `toml.workspace = true`
- `crates/tandem-core/src/ticket.rs` — struct changes, method replacements, helper removal, test updates

### No changes needed
- `tandem-storage` / `tandem-cli` — they call `to_canonical_toml()` which keeps the same public signature
- `Cargo.toml` (workspace root) — `toml = "1.0"` already declared as workspace dep

---

### Scope note
Validation confirms: (a) serde-derived TOML output differs cosmetically from current hand-rolled output (different blank-line placement, no blank lines after `title`/`priority`, `toml` crate inserts standard blank line before `[[documents]]`/`[table]` sections), (b) data is identical and both are valid TOML, (c) user accepts the cosmetic differences.

---

- [x] **Task 1**: Add `toml` dependency to `tandem-core`
  - File: `crates/tandem-core/Cargo.toml`
  - Change: Add `toml.workspace = true` to `[dependencies]` (after `serde.workspace = true`)
  - Verification: `cargo check -p tandem-core` (will fail initially since `toml` isn't imported yet — expected, unblocked by Task 2)

- [x] **Task 2**: Add `schema_version` fields to `TicketMeta` and `TicketState`, replace `to_canonical_toml()` bodies, add `#[serde(skip_serializing_if)]` on `document_fingerprints`
  - File: `crates/tandem-core/src/ticket.rs`
  - Changes:
    1. Add `use toml;` import (or just use `toml::to_string` inline)
    2. **`TicketMeta` struct**: Insert `pub schema_version: u8,` as the first field (before `id`)
    3. **`TicketMeta::new()`**: Add `schema_version: 1,` as the first field in the `Ok(Self { ... })` block
    4. **`TicketMeta::to_canonical_toml()`**: Replace entire body with:
       ```rust
       pub fn to_canonical_toml(&self) -> String {
           let mut sorted = self.clone();
           sorted.documents.sort_by(|a, b| a.name.cmp(&b.name));
           toml::to_string(&sorted).expect("TicketMeta serialization should not fail")
       }
       ```
    5. **`TicketState` struct**: Insert `pub schema_version: u8,` as the first field (before `status`), add `#[serde(skip_serializing_if = "BTreeMap::is_empty")]` on `document_fingerprints`
    6. **`TicketState::new()`**: Add `schema_version: 1,` as the first field in the `Ok(Self { ... })` block
    7. **`TicketState::to_canonical_toml()`**: Replace entire body with:
       ```rust
       pub fn to_canonical_toml(&self) -> String {
           toml::to_string(&self).expect("TicketState serialization should not fail")
       }
       ```
  - Verification: `cargo check -p tandem-core` (will compile once imports/syntax are correct)
  - Note: `toml::to_string` returns `Result`; `expect()` is safe because serialization of these simple types cannot fail

- [x] **Task 3**: Remove `toml_basic_string` and `toml_string_array` helper functions
  - File: `crates/tandem-core/src/ticket.rs`
  - Changes: Delete the two `fn` definitions (`toml_basic_string` at ~line 426, `toml_string_array` at ~line 443) and the `use std::fmt;` import if it becomes unused (check after removal — `#[derive(Debug)]` on enums uses `fmt::Debug` implicitly but `impl fmt::Display` still needs it)
  - Verification: `cargo check -p tandem-core` (no warnings about unused imports, no undefined references)

- [x] **Task 4**: Update unit test expectations for changed output format
  - File: `crates/tandem-core/src/ticket.rs` (the `#[cfg(test)] mod tests` section)
  - Changes update the `expected` string in these three exact-match assertions:
    1. **`meta_formats_as_canonical_toml`** (~line 623): Replace expected with:
       ```
       schema_version = 1
       id = "TNDM-4K7D9Q"
       title = "Add foo"
       type = "task"
       priority = "p2"
       depends_on = []
       tags = []

       [[documents]]
       name = "content"
       path = "content.md"
       ```
       (no blank lines after `title`/`priority`; blank line before `[[documents]]` from toml crate)
    2. **`meta_without_effort_canonical_toml_unchanged`** (~line 851): Same expected string as #1
    3. **`meta_with_effort_formats_as_canonical_toml`** (~line 876): Replace expected with:
       ```
       schema_version = 1
       id = "TNDM-4K7D9Q"
       title = "Add foo"
       type = "task"
       priority = "p2"
       effort = "m"
       depends_on = []
       tags = []

       [[documents]]
       name = "content"
       path = "content.md"
       ```
  - Tests that use `contains()` or position checks (`meta_canonical_toml_includes_documents`, `state_canonical_toml_includes_document_fingerprints`, `state_canonical_toml_omits_fingerprints_when_empty`, `documents_are_sorted_by_name_in_meta`, `state_formats_as_canonical_toml`) pass unchanged
  - Verification: `cargo test -p tandem-core` (all 30+ tests pass)

- [x] **Task 5**: Verify full workspace tests pass
  - Verification: `cargo test --workspace` (all tests across tandem-core, tandem-storage, tandem-cli pass)
  - Also verify no unused code warnings: `cargo check -p tandem-core --all-targets`
