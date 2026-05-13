## Context
`main.rs` is ~900 lines containing CLI parser defs, all command handlers, rendering, ID generation, and utilities.

## Suggestion
Split into modules by domain:
- `cli/mod.rs` — parser + dispatch
- `cli/ticket.rs` — ticket CRUD handlers
- `cli/doc.rs` — doc handler
- `cli/fmt.rs` — fmt handler
- `cli/awareness.rs` — awareness handler
- `cli/render.rs` — human output + highlighting
- `cli/util.rs` — ID generation, content loading

## First Task Before Planning
- [ ] **Validate this refactoring for correctness, best practices, and actual value to the project.** Check that the module boundaries don't create circular `pub(crate)` dependencies. Ensure the CLI binary size and compile time aren't negatively affected. Verify that all `#[allow(...)]` attributes remain on the right modules.
