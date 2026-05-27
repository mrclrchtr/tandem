# Task 9: Final verification: full test suite pass

## Goal

Run the full test suite to confirm all migrations are correct and no regressions were introduced.

## Verification steps

```bash
# Full workspace test suite
cargo test --workspace

# Also run clippy and fmt to ensure no warnings introduced
mise run clippy
mise run fmt
```

All tests must pass. No new warnings from clippy. No formatting violations.

If any test fails, fix the migration in the relevant file before proceeding.
