# Plan: Split tandem-cli main.rs into module tree

## Module structure

```
crates/tandem-cli/src/
├── main.rs              → stripped: only `#![allow(...)]`, `mod cli; fn main() { cli::run() }`
└── cli/
    ├── mod.rs           → Cli, Command, AwarenessArgs, OutputArgs, dispatch `run()`
    ├── util.rs          → generate_ticket_id, load_ticket_content, ticket_content_path, format_timestamp, constants
    ├── render.rs        → TicketJson*, print_ticket_human, SYNTAX_SET, THEME
    ├── ticket.rs        → TicketCommand, DocCommand, TicketDefinitionFilter, all ticket handlers
    ├── doc.rs           → handle_doc_create
    ├── fmt.rs           → handle_fmt
    └── awareness.rs     → handle_awareness, format_awareness_text
```

**Dependency graph** (no circular deps):
- `util.rs` — leaf, no module deps
- `render.rs` — depends on `util.rs` (format_timestamp)
- `ticket.rs` — depends on `render.rs` + `util.rs`, references `super::OutputArgs`
- `doc.rs` — depends on `util.rs`
- `fmt.rs` — depends on `util.rs`
- `awareness.rs` — depends on `util.rs`, references `super::AwarenessArgs`, `super::OutputArgs`
- `mod.rs` — depends on all submodules, dispatch orchestrator

## Implementation strategy

We use an **incremental extraction** approach: create `cli/` module tree and populate submodule files while keeping `main.rs`'s original code intact. Because Rust modules create separate namespaces, the new `pub(crate)` functions in submodules don't conflict with the private functions in `main.rs`. We then strip `main.rs` in the final step.

Verification per task: `cargo check`. Final verification: `cargo test` + `cargo clippy`.

---

- [x] **Task 1**: Create module skeleton — add `mod cli;` to main.rs, create `cli/mod.rs` with submodule declarations, create empty placeholder submodule files.
  - Files: `crates/tandem-cli/src/main.rs` (add `mod cli;`), `crates/tandem-cli/src/cli/mod.rs` (new), `crates/tandem-cli/src/cli/util.rs` (new), `crates/tandem-cli/src/cli/render.rs` (new), `crates/tandem-cli/src/cli/ticket.rs` (new), `crates/tandem-cli/src/cli/doc.rs` (new), `crates/tandem-cli/src/cli/fmt.rs` (new), `crates/tandem-cli/src/cli/awareness.rs` (new)
  - Main.rs: add `mod cli;` before `fn main()`. The original code stays intact.
  - `cli/mod.rs`: declare all 6 submodules with `mod util; mod render; mod ticket; mod doc; mod fmt; mod awareness;`
  - All submodule files: initially empty (Rust allows empty module files)
  - Verification: `cargo check` succeeds (main.rs unchanged, empty modules compile)

- [x] **Task 2**: Extract utility and render modules — populate `cli/util.rs` and `cli/render.rs` with extracted code from `main.rs`. All items get `pub(crate)` visibility. `main.rs` still has the original code (no conflict due to separate module namespaces).
  - **`cli/util.rs`**: extract constants (`CROCKFORD_BASE32`, `DEFAULT_CONTENT_TEMPLATE`) and functions (`generate_ticket_id`, `load_ticket_content`, `ticket_content_path`, `format_timestamp`). Required imports: `std::{env, fs, io::{self, Read}, path::PathBuf}`, `rand::RngExt`, `tandem_core::ticket::TicketId`, `tandem_storage::{FileTicketStore, TandemConfig, ticket_dir}`, `anyhow`.
  - **`cli/render.rs`**: extract JSON types (`TicketJsonEntry`, `TicketJson`, `TicketListJson`), `print_ticket_human`, and lazy statics (`SYNTAX_SET`, `THEME`). Required imports: `std::io::{self, IsTerminal}`, `std::sync::LazyLock`, `serde::Serialize`, `tandem_core::ticket::{Ticket, TicketId, TicketStatus}`, `syntect::*`, `termimad::*`, `time::*`. Uses `crate::cli::util::format_timestamp` via `super::util::format_timestamp`.
  - Verification: `cargo check` passes

- [x] **Task 3**: Extract handler modules — populate `cli/ticket.rs`, `cli/doc.rs`, `cli/fmt.rs`, `cli/awareness.rs` with extracted handler code.
  - **`cli/ticket.rs`**: extract `TicketDefinitionFilter` (ValueEnum), `TicketCommand` (Subcommand), `DocCommand` (Subcommand), and all ticket handler functions: `handle_ticket_create`, `handle_ticket_show`, `handle_ticket_list`, `handle_ticket_update`, `handle_ticket_sync`, `ticket_matches_definition_filter`. Preserve `#[allow(clippy::too_many_arguments)]` on create and update. Required imports: `std::{env, fs, io::{self, IsTerminal, Read}, path::PathBuf}`, `clap::{Args, Subcommand, ValueEnum}`, `serde::Serialize`, `tabled::*`, `tandem_core::ticket::*`, `tandem_storage::*`, `time::*`, `anyhow`. References `super::OutputArgs`.
  - **`cli/doc.rs`**: extract `handle_doc_create`. Required imports: `std::{env, fs}`, `tandem_core::ticket::*`, `tandem_storage::*`, `time::*`, `anyhow`, `serde_json`.
  - **`cli/fmt.rs`**: extract `handle_fmt`. Required imports: `std::{env, fs}`, `tandem_core::ticket::*`, `tandem_storage::*`, `anyhow`.
  - **`cli/awareness.rs`**: extract `handle_awareness` and `format_awareness_text`. Required imports: `std::env`, `tandem_core::awareness::*`, `tandem_repo::GitAwarenessProvider`, `tandem_storage::*`, `anyhow`. References `super::AwarenessArgs` and `super::OutputArgs`.
  - Verification: `cargo check` passes

- [x] **Task 4**: Wire up `cli/mod.rs` as the dispatch hub and strip `main.rs`.
  - **`cli/mod.rs`**: populate with `Cli`, `Command`, `AwarenessArgs`, `OutputArgs` types (from `clap` derives) and `pub(crate) fn run()` containing the dispatch match logic from `main()`.
  - Required imports: `clap::{Args, Parser, Subcommand}`, `std::path::PathBuf`. The `run()` function calls handlers via `ticket::handle_*`, `doc::handle_doc_create`, `fmt::handle_fmt`, `awareness::handle_awareness`.
  - **`main.rs`**: strip to `#![allow(clippy::disallowed_methods, clippy::disallowed_types)]`, `mod cli;`, and `fn main() -> anyhow::Result<()> { cli::run() }`.
  - Verification:
    1. `cargo check` succeeds
    2. `cargo test` (all integration tests pass — they invoke the binary via `Command::new(env!("CARGO_BIN_EXE_tndm"))`)
    3. `cargo clippy` produces no new warnings (crate-level `#![allow(...)]` preserved in main.rs, function-level `#[allow(...)]` preserved on handlers)

## Validation notes from design review

- **No circular dependencies**: verified in dependency graph above
- **Binary size / compile time**: module decomposition of a single crate has no material effect on either
- **`#[allow(...)]` attributes**:
  - Crate-level (`#![allow(clippy::disallowed_methods, clippy::disallowed_types)]`) stays in `main.rs`, applies to whole crate
  - Function-level (`#[allow(clippy::too_many_arguments)]`) preserved on `handle_ticket_create` and `handle_ticket_update` in `cli/ticket.rs`
