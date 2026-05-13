## Plan: Validate test-helper refactoring

### Files to modify
- `.tndm/tickets/TNDM-YAG308/content.md` — update with validation findings and decision

### Files to survey (read-only)
- `crates/tandem-core/src/awareness.rs` — the `ticket()` helper under `#[cfg(test)]`
- `crates/tandem-core/src/lib.rs` — current public exports
- `crates/tandem-core/Cargo.toml` — dev-dependencies
- `crates/tandem-storage/tests/ticket_store_tests.rs`
- `crates/tandem-storage/tests/create_ticket_reliability_tests.rs`
- `crates/tandem-storage/tests/awareness_storage_tests.rs`
- `crates/tandem-repo/tests/awareness_repo_tests.rs`
- `crates/tandem-cli/tests/ticket_cli_tests.rs`
- `crates/tandem-cli/tests/fmt_cli_tests.rs`
- `crates/tandem-cli/tests/awareness_cli_tests.rs`

### Tasks

- [x] **Task 1**: Complete duplication survey and evaluate alternatives
  - Action: Survey all test files across all 4 crates for ticket-builder patterns
  - Verify each usage category (same-module `#[cfg(test)]` helper, cross-crate test file, CLI-level raw file writer)
  - Document findings in the plan
  - Verification: All 10 test/lint files reviewed; duplication count documented

- [x] **Task 2**: Evaluate the three alternatives
  - **Option A — Do nothing**: Current state has a `ticket()` helper in exactly 1 file, used only within its own test module. No cross-crate duplication exists today.
  - **Option B — Feature-gated export from `tandem-core`**: Add `#[cfg(feature = "test-support")]` to a `pub mod test_helpers` in `tandem-core/lib.rs`. Standard Rust pattern (used by `proptest`, `quickcheck`). Adds negligible compile cost (feature flag, unlocked only when building tests in crates that depend on `tandem-core`).
  - **Option C — New `tandem-test-support` dev-dependency crate**: A 5th workspace crate for a single helper. Adds Cargo manifest, CI compilation cost, and `Cargo.lock` entry. Overkill for the current state.
  - Verification: Each alternative evaluated against: (a) current duplication count, (b) compile-time cost, (c) cross-crate usability, (d) maintenance burden

- [x] **Task 3**: Recommend and document the decision
  - Recommendation: **Option A (do nothing) for now**. The `ticket()` helper exists in one file and is used within its own crate. No cross-crate duplication exists. The ticket's own framing ("If helper duplication grows… this is lower priority") agrees — the threshold has not been met.
  - If/when a second crate duplicates a ticket builder, **Option B (feature-gated export)** is the appropriate response — thin, standard Rust idiom, no new workspace crate.
  - Update `content.md` with the validation findings and decision for future reference.
  - Verification: `content.md` contains the survey summary, alternatives analysis, and final recommendation

### Test exemptions
- All 3 tasks are test-exempt: they are investigation/documentation tasks with no code changes. Manual verification is via running `cargo test --workspace` to confirm no regressions (Task 3) and reviewing the survey output (Task 1).
