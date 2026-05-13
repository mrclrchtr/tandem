## Context
`ValidationError` (`tandem-core`), `StorageError` (`tandem-storage`), and `RepoError` (`tandem-repo`) are structurally identical: a wrapped `String` message with `new()`, `Display`, and `Error` impls.

## Suggestion
Define a shared error type in `tandem-core` (e.g., `TandemError`) and re-export or type-alias it in downstream crates. If crate-specific variants are needed later, wrap it in an enum without touching every call site.

## First Task Before Planning
- [ ] **Validate this refactoring for correctness, best practices, and actual value to the project.** Check that downstream crates don't rely on distinct error types for type-level dispatch. Ensure the shared type preserves `PartialEq` and `Clone` bounds. Consider whether this is premature abstraction given only 3 crates.
