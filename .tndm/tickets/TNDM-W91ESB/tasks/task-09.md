# Task 9: Final verification — full CI suite

Run the complete CI check suite to confirm the refactored code is identical in behavior to the original:

```bash
mise run check
```

This includes: `cargo fmt --check`, `cargo check --workspace`, `cargo xtask check-arch`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`.

All must pass with zero failures.

Additionally, confirm no regressions from the integration test suite:

```bash
cargo test --workspace -- --test-threads=1
```

All existing tests (unit + integration) must pass unchanged — the decomposition is pure structural, no logic modifications.
