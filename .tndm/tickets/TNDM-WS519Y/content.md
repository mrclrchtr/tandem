## Problem

Every handler in `ticket.rs` (11 functions) and `doc.rs` (1 function) repeats the same 4–6 line preamble:

```rust
let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
let store = FileTicketStore::new(repo_root);
let ticket_id = parse_ticket_id_input(&id, &config.id_prefix)?;
```

This is ~120 lines of duplicated boilerplate that makes handlers harder to scan and creates friction for new features.

## Approach

Extract a shared `TicketCtx` struct into a new file `crates/tandem-cli/src/cli/ticket_ctx.rs`.

```rust
pub(crate) struct TicketCtx {
    pub(crate) store: FileTicketStore,
    pub(crate) repo_root: PathBuf,
    config: TandemConfig,
}

impl TicketCtx {
    pub(crate) fn new() -> anyhow::Result<Self>;
    pub(crate) fn resolve_id(&self, input: &str) -> anyhow::Result<TicketId>;
    pub(crate) fn id_prefix(&self) -> &str;
}
```

## Files

| File | Action |
|------|--------|
| `crates/tandem-cli/src/cli/ticket_ctx.rs` | **New** — `TicketCtx` struct |
| `crates/tandem-cli/src/cli/ticket.rs` | Replace preamble in 11 handlers + helpers |
| `crates/tandem-cli/src/cli/doc.rs` | Replace preamble in `handle_doc_create` |
| `crates/tandem-cli/src/cli/mod.rs` | Add `mod ticket_ctx;` |

## Out of scope

- Consolidating duplicated tags/depends_on parsing in create/update
- Fixing `doc.rs` to use `store.update_ticket` instead of manual file writes
- Any behavior changes

## Verification

- `cargo build` compiles cleanly
- `mise run fmt` passes
- `mise run clippy` passes with no warnings
- `mise run test --workspace` all pass
- `mise run arch` workspace invariants hold
- Manual smoke test: `./tndm-dev ticket list`, `./tndm-dev ticket create "test"`
