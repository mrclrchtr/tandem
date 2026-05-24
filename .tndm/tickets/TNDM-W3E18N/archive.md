# Archive

## Verification Results

### Rust check suite (mise run check)
- cargo-check: ✔
- cargo-fmt: ✔
- cargo-clippy: ✔
- cargo xtask check-arch: ✔ (architecture checks passed)
- cargo test --workspace --locked: 170 passed, 0 failed

### PI plugin (TypeScript)
- tsc --noEmit: 0 errors
- vitest run: 47 passed, 0 failed

### Changes made

**Core model** (`crates/tandem-core/src/ticket/mod.rs`):
- Task struct stripped to `{number, title, status}`
- Removed: `files`, `verification`, `notes`, `detail_path`

**Core tests** (`state.rs`):
- Tests updated to match new Task shape

**Awareness** (`awareness.rs`):
- TaskSnapshotEntry stripped to `{number, title, status}`
- Old "task metadata diverged" test replaced with "task title diverged" test

**Storage** (`tandem-storage/src/lib.rs`):
- RawTask stripped; backward compatibility maintained (serde ignores unknown fields)

**CLI** (`crates/tandem-cli/src/cli/ticket.rs`):
- Removed `--file`, `--verification`, `--notes` from `task add`
- Removed `--file`, `--clear-files`, `--verification`, `--notes` from `task edit`
- Removed `normalize_task_files` function
- `prune_unlinked_canonical_task_detail_docs` now derives paths from canonical naming
- `detail_path` no longer set on Task struct (derived at convention level)

**CLI wiring** (`cli/mod.rs`):
- Updated handler signatures for changed task commands

**CLI tests** (`ticket_cli_tests.rs`):
- 6 tests fixed for removed fields; added `task_edit_rejects_replaced_args` test

**Storage tests** (`ticket_store_tests.rs`):
- Load test updated; TOML fixture retains old fields (backward compat verification)

**PI wrappers**:
- `tndm-cli.ts`: Removed `task_files`, `task_clear_files`, `task_verification`, `task_notes` from schema and executors; updated help text
- `flow-tools.ts`: Removed `files`, `clear_files`, `verification`, `clear_verification`, `notes`, `clear_notes` from schema, validation, and executors; `filterFlowTasks` now derives `detail_path` from canonical naming convention
- `extensions/index.ts`: Updated tool description text
- `flow-tools.test.ts`: Updated test fixtures and assertions
- `tndm-cli-tool.test.ts`: Removed obsolete `--clear-files` test

**README.md**:
- Updated task command descriptions
- Updated JSON examples to remove old fields
- Updated quick tour example to remove --file/--verification

**Review findings addressed**:
- #1 (PI wrappers pass removed flags): Fixed — schema, executors, descriptions, and tests all updated
- #2 (apply result loses task-detail context): Fixed — `filterFlowTasks` derives `detail_path` from canonical naming convention
- Review #2 findings #1 (schema still accepts removed params): Fixed — removed from schema
- Review #2 findings #2 (apply no longer exposes detail_path): Fixed — derived in filterFlowTasks
