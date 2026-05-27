# Task 4: Refactor doc.rs to use TicketCtx

In `crates/tandem-cli/src/cli/doc.rs`, replace the preamble in `handle_doc_create`:

```rust
// OLD (~5 lines):
let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
let store = FileTicketStore::new(repo_root.clone());
let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;

// NEW (2 lines):
let ctx = TicketCtx::new()?;
let ticket_id = ctx.resolve_id(&id)?;
```

Then replace:
- `store` → `ctx.store`
- `repo_root` → `ctx.repo_root`
- `repo_root.clone()` → `ctx.repo_root.clone()`

Add `use super::ticket_ctx::TicketCtx;` import. Remove unused imports from `std::env` and `tandem_storage::{discover_repo_root, load_config}`.

Note: `handle_doc_create` uses `FileTicketStore` for `load_ticket` — that goes through `ctx.store.load_ticket`.

**Verification**: `cargo build -p tandem-cli` compiles with zero errors.
