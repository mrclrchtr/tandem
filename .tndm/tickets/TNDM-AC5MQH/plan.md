# Plan: Extract ticket.rs into a module tree in tandem-core

## File structure (new)

```
crates/tandem-core/src/ticket/
  mod.rs    → string_enum! macro, TicketType, TicketPriority, TicketStatus, TicketEffort,
              NewTicket, Ticket, pub use re-exports, and associated tests
  id.rs     → TicketId + tests
  meta.rs   → TicketDocument, TicketMeta + tests
  state.rs  → TicketState + tests
```

**lib.rs** stays as `pub mod ticket;` — no change needed.

**Downstream compatibility:** All imports use `crate::ticket::*` or `tandem_core::ticket::*` — pub use re-exports in `mod.rs` fully preserve the flat import path. Verified across 7 files in tandem-storage, tandem-cli, and tandem-core itself.

---

## Tasks

- [x] **Task 1:** Create the `ticket/` module directory, move all content to `mod.rs`
  - Create `crates/tandem-core/src/ticket/` directory
  - Create `crates/tandem-core/src/ticket/mod.rs` with the identical content from `ticket.rs`
  - Delete `crates/tandem-core/src/ticket.rs`
  - Verification: `cargo test --workspace` passes

- [x] **Task 2:** Extract `TicketId` into `ticket/id.rs`
  - Create `crates/tandem-core/src/ticket/id.rs` with `TicketId` struct + impls + tests
  - Add `pub mod id;` and `pub use id::*;` in `mod.rs`
  - Remove moved code from `mod.rs`
  - Verification: `cargo test --workspace` passes

- [x] **Task 3:** Extract `TicketDocument` and `TicketMeta` into `ticket/meta.rs`
  - Create `crates/tandem-core/src/ticket/meta.rs` with `TicketDocument`, `TicketMeta` + impls + tests
  - Add `pub mod meta;` and `pub use meta::*;` in `mod.rs`
  - Remove moved code from `mod.rs`
  - Verification: `cargo test --workspace` passes

- [x] **Task 4:** Extract `TicketState` into `ticket/state.rs`
  - Create `crates/tandem-core/src/ticket/state.rs` with `TicketState` struct + impls + tests
  - Add `pub mod state;` and `pub use state::*;` in `mod.rs`
  - Remove moved code from `mod.rs`
  - Verification: `cargo test --workspace` passes

- [x] **Task 5:** Clean up `mod.rs` — final structure verification
  - `mod.rs` should retain: `string_enum!` macro, 4 enums, `NewTicket`, `Ticket`, `pub mod` + `pub use` for sub-modules, and enum/macro tests
  - Remove all orphaned code and update imports
  - Verification: `cargo test --workspace` + `cargo clippy --workspace` + `cargo xtask check-arch` all pass

- [x] **Task 6:** Canonical format verification
  - Test-exempt: pure internal module split, no behavioral change
  - Verification: `./tndm-dev fmt --check` succeeds
