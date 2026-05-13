## Context
A `ticket()` test helper exists in `tandem-core/src/awareness.rs` tests. Similar fixture builders may emerge in `tandem-repo` and `tandem-storage` tests.

## Suggestion
If helper duplication grows, create a `tandem-test-support` dev-dependency crate with fixture builders like `TicketBuilder`. For now duplication is minor (1–2 places), so this is lower priority.

## First Task Before Planning
- [ ] **Validate this refactoring for correctness, best practices, and actual value to the project.** Survey all test files to count actual duplication. Check whether a dev-dependency crate adds unacceptable compile-time overhead. Consider if `tandem-core`'s existing `#[cfg(test)]` modules can be exported for reuse instead.
