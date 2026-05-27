# Task 3: Refactor all handlers in ticket.rs to use TicketCtx

In `crates/tandem-cli/src/cli/ticket.rs`, replace the repetitive preamble in all 11 public handler functions:

- `handle_ticket_create`
- `handle_ticket_show`
- `handle_ticket_list`
- `handle_ticket_update`
- `handle_ticket_sync`
- `handle_task_add`
- `handle_task_list`
- `handle_task_complete`
- `handle_task_remove`
- `handle_task_edit`
- `handle_task_detail_ensure`
- `handle_task_set`

**Pattern for ID-using handlers:**
```rust
// OLD (~5 lines):
let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
let store = FileTicketStore::new(repo_root);
let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;

// NEW (2 lines):
let ctx = TicketCtx::new()?;
let ticket_id = ctx.resolve_id(&id)?;
```

**For `handle_ticket_list` (no ID resolution):**
```rust
// OLD (~4 lines):
let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
let store = FileTicketStore::new(repo_root);

// NEW (1 line):
let ctx = TicketCtx::new()?;
```

**For `handle_ticket_create` (custom ID generation):**
```rust
// OLD:
let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
let store = FileTicketStore::new(repo_root.clone());
let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
let ticket_id = match id {
    Some(value) => TicketId::parse(value)?,
    None => generate_ticket_id(&store, &config.id_prefix)?,
};
let content = load_ticket_content(content_file, content, stdin_content, &config)?;

// NEW:
let ctx = TicketCtx::new()?;
let ticket_id = match id {
    Some(value) => TicketId::parse(value)?,
    None => generate_ticket_id(&ctx.store, ctx.id_prefix())?,
};
let content = load_ticket_content(content_file, content, stdin_content, &ctx.config)?;
```

Then replace all remaining occurrences:
- `store` → `ctx.store`
- `repo_root` → `ctx.repo_root`
- `config.id_prefix` → `ctx.id_prefix()`
- `&config` → `&ctx.config`
- `repo_root.clone()` → `ctx.repo_root.clone()`

Also update `handle_task_add` which uses `store.clone()` for `FileTicketStore::new(repo_root.clone())` — replace with `ctx`. Add `use super::ticket_ctx::TicketCtx;` import at top of file.

Remove unused imports from `std::env` and `tandem_storage::{FileTicketStore, discover_repo_root, load_config}` that are no longer needed for preamble. Keep any used for other purposes.

**Verification**: `cargo build -p tandem-cli` compiles with zero errors.
