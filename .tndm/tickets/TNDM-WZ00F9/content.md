## Context
`TicketType`, `TicketPriority`, `TicketStatus`, `TicketEffort` in `crates/tandem-core/src/ticket.rs` each share ~35 lines of identical boilerplate: `parse()`, `as_str()`, `FromStr`, `Display`, `Serialize`, `Default`, and tests. This is ~200 lines of near-duplication.

## Suggestion
Define a small declarative macro inside `tandem-core` to collapse the 4× boilerplate into 1×. Existing tests verify each variant's round-trip, so a single macro test could suffice.

## First Task Before Planning
- [ ] **Validate this refactoring for correctness, best practices, and actual value to the project.** Consider whether a macro improves readability or hurts it. Check if `strum` or similar crates are already transitive dependencies. Ensure the macro doesn't obstruct rust-analyzer hover/docs.
