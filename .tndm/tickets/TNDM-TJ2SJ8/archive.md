# Archive

## Verification Results

### Fresh checks (2026-05-13)

- **`cargo check --workspace`**: 4 crates compiled, zero warnings, zero errors
- **`cargo test --workspace`**: 134 passed across 16 suites

### Changes made

| File | Change |
|------|--------|
| `crates/tandem-core/Cargo.toml` | Added `toml.workspace = true` |
| `crates/tandem-core/src/ticket.rs` | ~121 lines net removed |
| `crates/tandem-cli/tests/fmt_cli_tests.rs` | Updated 1 test expected string |

### Implementation details

- **`TicketMeta`** and **`TicketState`** both gained a `schema_version: u8` field (set to 1 in constructors) with `#[serde(skip)]`. The `#[serde(skip)]` is required because the CLI's `TicketJsonEntry` uses `#[serde(flatten)]` on both types simultaneously — without skipping, flattening would produce duplicate `schema_version` keys in JSON output.
- **`to_canonical_toml()`** now delegates to `toml::to_string()` and prepends `schema_version = 1\n` manually.
- Documents are sorted by name before serialization (preserving existing behavior).
- **`#[serde(skip_serializing_if = "BTreeMap::is_empty")]`** on `document_fingerprints` suppresses the empty `[document_fingerprints]` section (matching the original conditional logic).
- **`toml_basic_string`** and **`toml_string_array`** helper functions removed entirely — their escaping and array formatting is now handled by the `toml` crate.

### Task completion

- Task 1 ✅ — `toml` dependency added
- Task 2 ✅ — struct fields, `to_canonical_toml()` replaced, `skip_serializing_if` added
- Task 3 ✅ — helper functions removed
- Task 4 ✅ — core unit tests updated
- Task 5 ✅ — full workspace tests pass, zero warnings
