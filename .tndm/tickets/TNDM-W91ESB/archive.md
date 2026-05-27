# Archive

## Verification Evidence

### Fresh CI Suite (2026-05-27)
- **`cargo fmt --check`** — clean pass
- **`cargo check --workspace`** — 0 errors
- **`cargo clippy --workspace -- -D warnings`** — 0 warnings
- **`cargo test --workspace --locked`** — **193 passed, 1 ignored** (20 suites)
- **`cargo xtask check-arch`** — architecture checks passed

### Git Diff Summary
- **Deleted:** `crates/tandem-cli/src/cli/ticket.rs` (1,167 lines)
- **Modified:** `crates/tandem-cli/src/cli/mod.rs` (12 insertions, dispatch paths updated)
- **Created:** 7 new files under `crates/tandem-cli/src/cli/ticket/` (1,003 lines total)
  - `mod.rs` — command enums
  - `create.rs` — create handler + args
  - `list.rs` — list handler + args + definition filter
  - `show.rs` — show handler
  - `sync.rs` — sync handler
  - `update.rs` — update handler + TicketUpdate struct + tests
  - `task.rs` — all 7 task handlers + helpers

### Doc Updates
- `CLAUDE.md` — updated "Adding a new optional field to TicketMeta" steps to reflect new file locations (create.rs, update.rs split)
- `ticket_ctx.rs` — updated doc comment referencing old monolithic file

### Behavior
Zero logic changes — pure structural decomposition. All existing tests pass without modification.
