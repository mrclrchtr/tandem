## Plan: Decompose ticket.rs into focused CLI sub-modules

### Scope check

Single coherent refactoring — no need to split across plans.

### Execution strategy

Create all new modules under `ticket/` first, verify they compile, then update `cli/mod.rs` dispatches and delete the old `ticket.rs`. No logic changes anywhere.

### File map

| File | Created/Modified | Responsibility |
|------|-----------------|----------------|
| `crates/tandem-cli/src/cli/ticket/mod.rs` | Created | `TicketCommand`, `DocCommand`, `TaskCommand`, `TaskDetailCommand` enums + public re-exports |
| `crates/tandem-cli/src/cli/ticket/show.rs` | Created | `handle_ticket_show` |
| `crates/tandem-cli/src/cli/ticket/sync.rs` | Created | `handle_ticket_sync` |
| `crates/tandem-cli/src/cli/ticket/list.rs` | Created | `TicketListArgs`, `TicketDefinitionFilter`, `handle_ticket_list`, `ticket_matches_definition_filter` |
| `crates/tandem-cli/src/cli/ticket/update.rs` | Created | `TicketUpdateArgs`, `TicketUpdate` struct + methods, `handle_ticket_update`, unit tests |
| `crates/tandem-cli/src/cli/ticket/create.rs` | Created | `TicketCreateArgs`, `handle_ticket_create`, unit tests (imports `TicketUpdate` from `ticket/update.rs`) |
| `crates/tandem-cli/src/cli/ticket/task.rs` | Created | Task enums + all `handle_task_*` functions + internal helpers |
| `crates/tandem-cli/src/cli/mod.rs` | Modified | Update `mod ticket` to `mod ticket;` + update match dispatch paths |
| `crates/tandem-cli/src/cli/ticket.rs` | Deleted | Replaced by `ticket/` directory |

### Cross-module dependencies

- `ticket/create.rs` → `ticket/update.rs` (for `TicketUpdate::from_create_args`, a pure constructor)
- `ticket/task.rs` → `../doc.rs` (for `recompute_ticket_document_fingerprints`, unchanged)
- All modules → `../ticket_ctx.rs`, `../render.rs`, `../util.rs`, `tandem_core`, `tandem_storage` (unchanged)

### Verification gates

Each task includes its own verification. The final task runs the full CI suite (build, test, clippy, fmt, arch).
