# Archive

## Verification Results (fresh)

### Tests
- `cargo test --workspace --locked` → **123 passed, 1 ignored** (16 suites, 2.44s)
- All existing serialization tests pass, confirming JSON output is identical:
  - `macro_generated_impls` checks serde_json output for all 4 enums
  - `ticket_meta_serializes_with_type_renamed` checks JSON shape
  - `ticket_state_serializes_all_fields` checks JSON shape
  - `meta_formats_as_canonical_toml` / `meta_without_effort_canonical_toml_unchanged` / `meta_with_effort_formats_as_canonical_toml` check TOML output
  - `ticket_meta_serializes_effort_as_null_when_absent` / `ticket_meta_serializes_effort_when_present` check JSON for optional effort field

### Lint
- `cargo clippy --workspace --locked` → No issues found

### Architecture
- `cargo xtask check-arch` → architecture checks passed

### Change summary
Modified `crates/tandem-core/src/ticket.rs`:
- Added `#[derive(serde::Serialize)]` and `#[serde(rename_all = "snake_case")]` to the enum definition inside `string_enum!` macro
- Removed the manual `impl serde::Serialize for $name { ... }` block

The `snake_case` variant → string mapping was verified for all 19 variants across 4 enums and matches the original `$str` literals exactly. The `rename_all = "snake_case"` pattern was already established in `awareness.rs` (`AwarenessChangeKind`). No new dependencies added.
