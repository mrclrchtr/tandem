## Plan: Eliminate duplicated DEFAULT_CONTENT_TEMPLATE

### Approach
Move `DEFAULT_CONTENT_TEMPLATE` into `tandem-core` as `tandem_core::ticket::DEFAULT_CONTENT_TEMPLATE`, then update all consumers to reference it instead of their own local constants. Four copies exist (2 source, 2 test) — all become one canonical definition.

### Files to modify
1. `crates/tandem-core/src/ticket.rs` — add the canonical constant
2. `crates/tandem-storage/src/lib.rs` — replace inline `concat!()` with reference to core constant
3. `crates/tandem-cli/src/cli/util.rs` — replace local constant with reference to core constant
4. `crates/tandem-storage/tests/config_tests.rs` — replace local constant with reference to core constant
5. `crates/tandem-cli/tests/ticket_cli_tests.rs` — replace local constant with reference to core constant

---

- [x] **Task 1**: Add `DEFAULT_CONTENT_TEMPLATE` constant to `tandem-core/src/ticket.rs`
  - File: `crates/tandem-core/src/ticket.rs`
  - Add before the `#[cfg(test)]` module: `pub const DEFAULT_CONTENT_TEMPLATE: &str = concat!( ... )` with the exact template string content (same string as currently duplicated across 4 files)
  - Verification: `cargo build --package tandem-core` compiles cleanly

- [x] **Task 2**: Update `tandem-storage/src/lib.rs` to reference the core constant
  - File: `crates/tandem-storage/src/lib.rs`
  - In `TandemConfig::default()`, replace the inline `concat!(...).to_string()` with `tandem_core::ticket::DEFAULT_CONTENT_TEMPLATE.to_string()`
  - Verification: `cargo build --package tandem-storage` compiles cleanly

- [x] **Task 3**: Update `tandem-cli/src/cli/util.rs` to reference the core constant
  - File: `crates/tandem-cli/src/cli/util.rs`
  - Remove the local `pub(crate) const DEFAULT_CONTENT_TEMPLATE` definition
  - Keep the `load_ticket_content` function behavior: the fallback to `config.content_template` then to the core constant still works because `TandemConfig::default()` already provides the template (so the fallthrough only happens if user explicitly sets `content_template = ""`)
  - Change the last line of `load_ticket_content` from `Ok(DEFAULT_CONTENT_TEMPLATE.to_string())` to `Ok(tandem_core::ticket::DEFAULT_CONTENT_TEMPLATE.to_string())`
  - Verification: `cargo build --package tandem-cli` compiles cleanly

- [x] **Task 4**: Update `tandem-storage/tests/config_tests.rs` to reference the core constant
  - File: `crates/tandem-storage/tests/config_tests.rs`
  - Remove the local `const DEFAULT_CONTENT_TEMPLATE` definition
  - Use `tandem_core::ticket::DEFAULT_CONTENT_TEMPLATE` directly in the assertion
  - Verification: `cargo test --package tandem-storage --test config_tests` passes

- [x] **Task 5**: Update `tandem-cli/tests/ticket_cli_tests.rs` to reference the core constant
  - File: `crates/tandem-cli/tests/ticket_cli_tests.rs`
  - Remove the local `const DEFAULT_CONTENT_TEMPLATE` definition
  - Use `tandem_core::ticket::DEFAULT_CONTENT_TEMPLATE` directly in the assertion
  - Verification: `cargo test --package tandem-cli --test ticket_cli_tests` passes

- [x] **Task 6**: Run full workspace verification
  - Verification: `cargo xtask check-arch && cargo clippy --workspace --locked && cargo test --workspace --locked` all pass
