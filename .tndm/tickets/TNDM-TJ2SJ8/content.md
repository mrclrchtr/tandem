## Context
`TicketMeta::to_canonical_toml()` and `TicketState::to_canonical_toml()` manually concatenate strings. The `toml_basic_string` helper doesn't escape all TOML special characters (e.g., backspace, form feed), and document sorting is ad-hoc. `tandem-storage` already depends on `toml`, but `tandem-core` does not.

## Suggestion
Move `toml` into `tandem-core`'s dependencies and derive `Serialize` for `TicketMeta` / `TicketState`, using custom serializers for `TicketId` (string) and field reordering via `#[serde(rename)]`. Eliminate `toml_basic_string` / `toml_string_array` helpers.

## First Task Before Planning
- [ ] **Validate this refactoring for correctness, best practices, and actual value to the project.** Check that the derived TOML output is byte-for-byte identical to current canonical output. Verify that `toml` crate dependency in core is acceptable. Ensure serde attributes don't complicate the public API.
