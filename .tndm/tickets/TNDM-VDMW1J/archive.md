# Archive

## Verification Results

### Fresh checks (all passed)
- `cargo xtask check-arch` — ✅ architecture checks passed
- `cargo clippy --workspace --locked` — ✅ no issues found
- `cargo test --workspace --locked` — ✅ 123 passed, 1 ignored

### Implementation summary
The `DEFAULT_CONTENT_TEMPLATE` string was moved from 4 duplicated locations into a single canonical constant `tandem_core::ticket::DEFAULT_CONTENT_TEMPLATE`:

| File | Change |
|------|--------|
| `crates/tandem-core/src/ticket.rs` | Added canonical constant (+20 lines) |
| `crates/tandem-storage/src/lib.rs` | Replaced inline `concat!()` with reference (-18 lines) |
| `crates/tandem-cli/src/cli/util.rs` | Removed local constant, uses core in `load_ticket_content()` fallback (-19 lines) |
| `crates/tandem-storage/tests/config_tests.rs` | Removed local constant, references core (-19 lines) |
| `crates/tandem-cli/tests/ticket_cli_tests.rs` | Removed local constant, references core (-19 lines) |

**Total: +24 / -75 lines across 4 source/test files.**

### Design intent verified
- ✅ Template is no longer duplicated across the codebase
- ✅ Template remains overridable via `[templates] content` in `.tndm/config.toml`
- ✅ No new dependencies created (tandem-core is the architecture root, already depended on by both storage and CLI)
- ✅ No behavior change — same string, same fallback paths, same override mechanism
