## Context
`DEFAULT_CONTENT_TEMPLATE` exists in `tandem-cli::main.rs` and `TandemConfig::default()` in `tandem-storage`. They are byte-for-byte identical strings.

## Suggestion
Move the template string to `tandem-core` as a constant (e.g., `tandem_core::DEFAULT_CONTENT_TEMPLATE`), or have `tandem-cli` use `TandemConfig::default().content_template` instead of its own constant.

## First Task Before Planning
- [ ] **Validate this refactoring for correctness, best practices, and actual value to the project.** Verify that removing the CLI-level constant doesn't break build-time compilation or require `tandem-storage` as a new CLI dependency. Ensure the template remains overridable via `config.toml`.
