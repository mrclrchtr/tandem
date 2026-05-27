# Task 1: Create TicketCtx struct in new ticket_ctx.rs

Create `crates/tandem-cli/src/cli/ticket_ctx.rs` with:

- `TicketCtx` struct holding `pub(crate) store: FileTicketStore`, `pub(crate) repo_root: PathBuf`, and `config: TandemConfig` (private)
- `TicketCtx::new()` — runs `env::current_dir → discover_repo_root → load_config → FileTicketStore::new`, returns `anyhow::Result<Self>`
- `TicketCtx::resolve_id(&self, input: &str) -> anyhow::Result<TicketId>` — delegates to `parse_ticket_id_input`
- `TicketCtx::id_prefix(&self) -> &str` — accessor for config prefix

Use imports: `std::env`, `std::path::PathBuf`, `tandem_core::ticket::TicketId`, `tandem_storage::{FileTicketStore, discover_repo_root, load_config}`, and `super::util::parse_ticket_id_input`.

No tests needed — this is a pure extraction delegate with zero new logic.

**Verification**: `cargo build -p tandem-cli` compiles (even if unused).
