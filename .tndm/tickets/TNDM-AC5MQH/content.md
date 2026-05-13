## Context
`ticket.rs` is 800+ lines and contains `TicketId`, `TicketMeta`, `TicketState`, `TicketDocument`, `NewTicket`, `Ticket`, plus TOML helpers and unit tests.

## Suggestion
Split into:
- `ticket/id.rs`
- `ticket/meta.rs`
- `ticket/state.rs`
- `ticket/mod.rs` (re-exports + `NewTicket`/`Ticket` structs)

## First Task Before Planning
- [ ] **Validate this refactoring for correctness, best practices, and actual value to the project.** Consider whether the module overhead improves navigation or just adds indirection. Check if any downstream crates rely on specific `ticket.rs` paths. Ensure `pub use` re-exports preserve the existing API surface.
