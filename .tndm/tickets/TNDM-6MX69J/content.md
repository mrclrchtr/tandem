## Context
The `no_explicit_create` / `no_explicit_update` guard + stdin read is copy-pasted between `handle_ticket_create` and `handle_ticket_update` in `tandem-cli`.

## Suggestion
Extract a small helper:
```rust
fn read_stdin_if_no_flags(no_explicit: bool) -> anyhow::Result<Option<String>>;
```
Or compute the boolean once and pass it in.

## First Task Before Planning
- [ ] **Validate this refactoring for correctness, best practices, and actual value to the project.** Ensure the helper preserves the exact TTY-detection and empty-buffer semantics. Check that error messages stay identical for both commands. Consider whether a shared helper is worth it for only two call sites.
