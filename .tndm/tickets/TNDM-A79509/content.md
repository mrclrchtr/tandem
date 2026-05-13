## Context
Each enum (`TicketType`, `TicketPriority`, `TicketStatus`, `TicketEffort`) has a ~4-line manual `serde::Serialize` impl that calls `serializer.serialize_str(...)`.

## Suggestion
If not macro-ifying the enums entirely, use a single `serialize_with` helper or `serde_repr` to remove the manual impls. Example:
```rust
#[derive(Serialize)]
#[serde(serialize_with = "serialize_as_str")]
enum TicketType { ... }
```

## First Task Before Planning
- [ ] **Validate this refactoring for correctness, best practices, and actual value to the project.** Ensure the helper works for all 4 enums without adding new dependencies. Verify that JSON output remains identical. Check whether `serde_repr` is already in the dependency tree.
