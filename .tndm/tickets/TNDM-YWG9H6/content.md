# Refactor: break up load_ticket, deduplicate task_detail_ensure, extract task validation

Three pure internal refactors in the tandem Rust workspace. No behavior changes, no API surface changes. All covered by existing tests.

## 1. Break up `load_ticket()` in `tandem-storage` (`src/lib.rs:389–551`)

**Problem**: `load_ticket` is ~160 lines of monolithic code mixing file I/O, TOML deserialization, field-by-field validation, legacy migration, and domain-type construction into one flat function. It has grown organically with each new field addition and is the hardest function in the codebase to extend or unit-test in isolation.

**Approach**: Extract two private helper functions:

- `parse_meta(raw: RawTicketMeta, id: &TicketId, meta_path: &Path) -> Result<TicketMeta>` — handles depends_on parsing/sorting, tags sorting, field-by-field validation, and legacy document migration.
- `parse_state(raw: RawTicketState, state_path: &Path) -> Result<TicketState>` — handles status parsing, document fingerprints, and task array construction.

`load_ticket` becomes a thin orchestrator: read three files → deserialize raw types → delegate to helpers → assemble `Ticket`. The existing error messages and paths are preserved exactly.

**Files**: `crates/tandem-storage/src/lib.rs` only.

## 2. Deduplicate remaining path computation in `handle_task_detail_ensure` (`ticket.rs:873–923`)

**Problem**: The recent refactor (commit `fff24e3`) extracted `ensure_canonical_task_detail_doc`, but `handle_task_detail_ensure` discards the returned `rel_path` and recomputes it — along with `abs_path` — via a redundant call to `canonical_task_detail_doc(number)` + `ticket_dir()`. The `ensure_canonical_task_detail_doc` helper already computes and returns `(rel_path, created_file)`.

**Approach**: Capture the returned `rel_path` instead of discarding it with `_`. Compute `abs_path` from it rather than recomputing from scratch. Keep the `doc_name` extraction (needed for JSON output and is a pure format-string computation, not duplication).

**Files**: `crates/tandem-cli/src/cli/ticket.rs` only.

## 3. Extract `validate_tasks()` to `tandem-core` (`ticket/mod.rs`)

**Problem**: Task validation rules (number ≥ 1, uniqueness, non-empty title) are inlined inside `handle_task_set`. `handle_task_add` also validates non-empty titles separately. As task operations grow, scattered validation risks inconsistency.

**Approach**: Add a public `validate_tasks(tasks: &[Task]) -> Result<(), ValidationError>` function in `tandem-core` that checks: numbers ≥ 1, no duplicate numbers, no empty titles. Call it from `handle_task_set` and `handle_task_add`. The function returns a `ValidationError` with clear messages. Also call it from `handle_task_edit` where title is changed to a non-empty value (already validated inline, but now centralized).

**Files**: `crates/tandem-core/src/ticket/mod.rs` (add function + tests), `crates/tandem-cli/src/cli/ticket.rs` (replace inline validation with call).

## Constraints

- No behavior changes — all existing tests must pass unchanged
- Existing error messages preserved
- Public API of all crates unchanged
- Architecture boundaries preserved (tandem-core remains IO-free, only tandem-cli depends on clap)

## Verification

- `mise run check` (fmt + compile + arch + clippy)
- `mise run test` (full test suite)
- `./tndm-dev fmt --check` (canonical TOML verification)
