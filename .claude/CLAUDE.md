## Code Intelligence

Prefer language-server tools over grep or broad file reads for code navigation. Start with symbol-aware lookup, then read only the smallest relevant scope.

Default tool order:
- `workspaceSymbol` for workspace-wide symbol discovery
- `documentSymbol` for file structure
- `goToDefinition` for source definitions
- `goToImplementation` for concrete implementations
- `findReferences` for usages and impact analysis
- `hover` for types, signatures, and docs
- `prepareCallHierarchy` + `incomingCalls` / `outgoingCalls` for call-flow analysis

Rules:
- Do not start with grep or full-file reads if an LSP tool can answer the question.
- Use `documentSymbol` before reading large files.
- Use references and call hierarchy to understand impact before editing.
- Read incrementally: symbol first, then local context, then broader context only if required.
- Use text search only for comments, strings, logs, config, generated text, or when LSP support is unavailable/unreliable.

After edits:
- Check LSP diagnostics immediately.
- If diagnostics show errors after a commit, verify with `cargo clippy` first — rust-analyzer frequently lags and shows false positives on recently committed code.
- Fix introduced errors and warnings before proceeding.
- If diagnostics are unavailable, use `cargo clippy` as the authoritative fallback.
