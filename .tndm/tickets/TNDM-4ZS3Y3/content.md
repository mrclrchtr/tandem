## Context
The SHA-256 `sha256:<hex>` formatting logic is duplicated in 3 places:
1. `FileTicketStore::fingerprint_file` (`tandem-storage`)
2. `FileTicketStore::create_ticket` (`tandem-storage`)
3. `tandem-cli::handle_doc_create` (`tandem-cli`)

## Suggestion
Move `fingerprint_file` and a `fingerprint_bytes` variant to a small public utility in `tandem-storage` or `tandem-core`. Replace all inline hashing with these helpers.

## First Task Before Planning
- [ ] **Validate this refactoring for correctness, best practices, and actual value to the project.** Verify that the extracted helper produces identical output in all 3 call sites. Check whether `tandem-core` or `tandem-storage` is the right home. Ensure error types from file IO remain appropriate for each caller.
