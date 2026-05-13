## Plan: Replace manual Serialize impl with derive + `rename_all`

### Files to modify

- `crates/tandem-core/src/ticket.rs` — the `string_enum!` macro

- [x] **Task 1**: Modify the `string_enum!` macro to derive `Serialize` with `rename_all = "snake_case"` instead of a manual impl
  - File: `crates/tandem-core/src/ticket.rs`
  - Change: Remove the manual `impl serde::Serialize for $name { ... }` block from the macro. Add `#[derive(serde::Serialize)]` and `#[serde(rename_all = "snake_case")]` to the enum definition (after the outer `$(#[$attr])*`).
  - Verification: `cargo test --workspace --locked` — all existing tests must pass, especially `macro_generated_impls` which verifies serde JSON serialization for all 4 enums. Also `cargo clippy --workspace --locked`.

- [x] **Task 2**: Verify serialization output is unchanged
  - File: `crates/tandem-core/src/ticket.rs` (tests already cover this)
  - Verification: No code changes — existing `macro_generated_impls` test already checks `serde_json::to_string` output for each enum.

- [x] **Task 3**: Run full validation suite
  - Verification: `cargo xtask check-arch` passes and no architecture rules are violated.
