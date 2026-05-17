# Implementation Plan: Structured Task Management in TNDM

## File Map

| File | Role |
|---|---|
| `crates/tandem-core/src/ticket/mod.rs` | `TaskStatus` enum, `Task` struct, `string_enum!` macro use |
| `crates/tandem-core/src/ticket/state.rs` | `tasks: Vec<Task>` field on `TicketState` |
| `crates/tandem-core/src/awareness.rs` | `AwarenessFieldDiffs.tasks` for task-aware diffs |
| `crates/tandem-storage/src/lib.rs` | `RawTask`/`RawTaskStatus` deserialization, `[[tasks]]` in state TOML |
| `crates/tandem-cli/src/cli/ticket.rs` | `TicketTaskCommand` (add/list/complete/remove/edit/set) and handlers |
| `crates/tandem-cli/src/cli/mod.rs` | Dispatch `TicketCommand::Task` to task handlers |
| `crates/tandem-cli/src/cli/render.rs` | JSON output already flattens `TicketState` — verify tasks appear |
| `plugins/supi-flow/extensions/tools/tndm-cli.ts` | New `task_add`, `task_list`, `task_complete`, `task_remove`, `task_edit`, `task_set` actions |
| `plugins/supi-flow/extensions/tools/flow-tools.ts` | Rewrite `executeFlowPlan` (parse markdown → `task set`), `executeFlowCompleteTask` (→ `task complete` CLI) |
| `plugins/supi-flow/extensions/index.ts` | Update tool descriptions to reflect task-based behavior |
| `plugins/supi-flow/skills/supi-flow-plan/SKILL.md` | Replace plan.md references with task-based description |
| `plugins/supi-flow/skills/supi-flow-apply/SKILL.md` | Replace plan.md references with task-based description |
| `plugins/supi-flow/skills/supi-flow-archive/SKILL.md` | Replace plan.md references with task-based description |
| `plugins/supi-flow/__tests__/flow-tools.test.ts` | Update tests for new flow tool behavior |
| `crates/tandem-cli/tests/ticket_cli_tests.rs` | Integration tests for `tndm ticket task` subcommands |

---

- [x] **Task 1**: Add `TaskStatus` enum and `Task` struct to `tandem-core`
  - File: `crates/tandem-core/src/ticket/mod.rs`
  - Use `string_enum!` macro for `TaskStatus` with variants `Todo => "todo"` and `Done => "done"` (no Default — the enum only has two states, no sensible default; the `new` constructor or CLI will set initial status explicitly)
  - Define `Task` struct:
    ```rust
    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct Task {
        pub number: u32,
        pub title: String,
        pub status: TaskStatus,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub file: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub verification: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub notes: Option<String>,
    }
    ```
  - Add unit tests in `mod tests`: parse/as_str roundtrip for `TaskStatus`, JSON serialization for `Task` with all fields, JSON serialization with only required fields
  - Verification: `cargo test -p tandem-core -- ticket`

- [x] **Task 2**: Add `tasks` field to `TicketState`
  - File: `crates/tandem-core/src/ticket/state.rs`
  - Add field: `#[serde(default, skip_serializing_if = "Vec::is_empty")] pub tasks: Vec<Task>`
  - Update `TicketState::new()` to initialize `tasks: Vec::new()`
  - Add a `TaskStatus` import
  - Update existing tests that check canonical TOML output — the empty tasks vec is skipped, so existing `to_canonical_toml()` tests should be unaffected
  - Add new test: `state_canonical_toml_includes_tasks()` — creates state with tasks and verifies the `[[tasks]]` TOML section appears
  - Add new test: `state_serializes_tasks_to_json()` — verifies JSON roundtrip with tasks
  - Verification: `cargo test -p tandem-core -- state`

- [x] **Task 3**: Add `tasks` serialization/deserialization in `tandem-storage`
  - File: `crates/tandem-storage/src/lib.rs`
  - Add `RawTaskStatus` string enum (deserialize from `"todo"`/`"done"`):
    ```rust
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum RawTaskStatus { Todo, Done }
    ```
  - Add `RawTask` deserialization struct:
    ```rust
    #[derive(Debug, Deserialize)]
    struct RawTask {
        number: u32,
        title: String,
        status: RawTaskStatus,
        file: Option<String>,
        verification: Option<String>,
        notes: Option<String>,
    }
    ```
  - Add `tasks: Option<Vec<RawTask>>` to `RawTicketState`
  - In `load_ticket`, parse tasks from raw state into `state.tasks`:
    ```rust
    state.tasks = raw_state.tasks.unwrap_or_default().into_iter().map(|raw| Task {
        number: raw.number,
        title: raw.title,
        status: match raw.status { RawTaskStatus::Todo => TaskStatus::Todo, RawTaskStatus::Done => TaskStatus::Done },
        file: raw.file,
        verification: raw.verification,
        notes: raw.notes,
    }).collect();
    ```
  - Add test: `load_ticket_with_tasks()` in `ticket_store_tests.rs` — creates a ticket with tasks, writes state.toml with `[[tasks]]`, loads and verifies
  - Add test: `load_ticket_without_tasks_is_empty_vec()` — creates a ticket without `[[tasks]]` in TOML, loads and verifies `tasks` is empty vec (backward compat)
  - Verification: `cargo test -p tandem-storage`

- [x] **Task 4**: Add tasks diff to awareness (`tandem-core`)
  - File: `crates/tandem-core/src/awareness.rs`
  - Add `tasks: Option<AwarenessTasksDiff>` to `AwarenessFieldDiffs`
  - Define `AwarenessTasksDiff`:
    ```rust
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct AwarenessTasksDiff {
        pub current: Vec<TaskSnapshotEntry>,
        pub against: Vec<TaskSnapshotEntry>,
    }
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct TaskSnapshotEntry {
        pub number: u32,
        pub title: String,
        pub status: String,
    }
    ```
  - In `AwarenessFieldDiffs::between()`, compare `current.state.tasks` vs `against.state.tasks`, doing a task-by-task diff (compare by number, check if title/status changed)
  - Only include `tasks` diff if the task lists differ (via `is_empty()` check)
  - Update `is_empty()` to include `self.tasks.is_none()`
  - Add tests: identical tasks produce no diff, changed task status produces diff, added/removed task produces diff
  - Verification: `cargo test -p tandem-core -- awareness`

- [x] **Task 5**: Add `tndm ticket task` CLI subcommands
  - Files: `crates/tandem-cli/src/cli/ticket.rs`, `crates/tandem-cli/src/cli/mod.rs`
  - Add `TaskCommand` enum with subcommands:
    ```rust
    #[derive(Subcommand, Debug)]
    pub(crate) enum TaskCommand {
        /// Add a task to a ticket.
        Add {
            id: String,
            #[arg(long, short = 't')] title: String,
            #[arg(long, short = 'f')] file: Option<String>,
            #[arg(long, short = 'v')] verification: Option<String>,
            #[arg(long, short = 'n')] notes: Option<String>,
            #[command(flatten)] output: OutputArgs,
        },
        /// List tasks for a ticket.
        List {
            id: String,
            #[command(flatten)] output: OutputArgs,
        },
        /// Mark a task as done.
        Complete {
            id: String,
            number: u32,
            #[command(flatten)] output: OutputArgs,
        },
        /// Remove a task from a ticket.
        Remove {
            id: String,
            number: u32,
            #[command(flatten)] output: OutputArgs,
        },
        /// Edit a task's fields.
        Edit {
            id: String,
            number: u32,
            #[arg(long, short = 't')] title: Option<String>,
            #[arg(long, short = 'f')] file: Option<String>,
            #[arg(long, short = 'v')] verification: Option<String>,
            #[arg(long, short = 'n')] notes: Option<String>,
            #[command(flatten)] output: OutputArgs,
        },
        /// Bulk-replace all tasks from JSON input.
        Set {
            id: String,
            #[arg(long)] tasks: String,
            #[command(flatten)] output: OutputArgs,
        },
    }
    ```
  - Add `TicketCommand::Task { #[command(subcommand)] command: TaskCommand }` variant to `TicketCommand`
  - Implement handlers:
    - `handle_task_add`: load ticket, auto-assign number (max existing + 1 or 1), push new `Task` with `status: TaskStatus::Todo`, bump revision, persist
    - `handle_task_list`: load ticket, if `--json`: print JSON array of tasks; else: print human-readable table (number, status, title)
    - `handle_task_complete`: load ticket, find task by number, set `status = TaskStatus::Done`, bump revision, persist; error if not found
    - `handle_task_remove`: load ticket, find task by number, remove from vec, bump revision, persist; error if not found
    - `handle_task_edit`: load ticket, find task by number, update provided fields, bump revision, persist; error if not found
    - `handle_task_set`: load ticket, parse `--tasks` JSON string into `Vec<Task>`, replace `state.tasks`, bump revision, persist; validate task numbers are 1-based and unique
  - Each handler bumps `state.revision` and `state.updated_at`
  - In `mod.rs`, dispatch `TicketCommand::Task { command }` to the appropriate handler
  - Verification: `cargo build -p tandem-cli && ./tndm-dev ticket task --help`

- [x] **Task 6**: Add integration tests for `tndm ticket task` CLI
  - File: `crates/tandem-cli/tests/ticket_cli_tests.rs`
  - Tests to add:
    - `task_add_creates_task_with_auto_number`: add task, verify JSON output includes task with number=1 and status=todo
    - `task_add_increments_number`: add two tasks, verify second has number=2
    - `task_list_json_output`: add tasks, list --json, verify array of task objects
    - `task_complete_marks_task_done`: add task, complete number 1, show --json and verify status=done
    - `task_complete_nonexistent_fails`: complete number 99 on empty ticket, verify error
    - `task_remove_deletes_task`: add two tasks, remove number 1, verify only number 2 remains
    - `task_edit_updates_fields`: add task, edit title/file/verification/notes, verify changes
    - `task_set_bulk_replace`: add tasks, set --tasks with new JSON array, verify old tasks replaced
    - `task_set_empty_clears`: add tasks, set --tasks '[]', verify all tasks removed
    - `task_set_duplicate_numbers_fails`: set with duplicate task numbers, verify error
  - Verification: `cargo test -p tandem-cli -- ticket_cli_tests`

- [x] **Task 7**: Add task actions to `supi_tndm_cli` TypeScript tool
  - File: `plugins/supi-flow/extensions/tools/tndm-cli.ts`
  - Extend `actionEnum` to include `"task_add"`, `"task_list"`, `"task_complete"`, `"task_remove"`, `"task_edit"`, `"task_set"`
  - Add params to `supi_tndm_cli_params`:
    ```typescript
    task_title: Type.Optional(Type.String({ description: "Task title (required for task_add)" })),
    task_number: Type.Optional(Type.Number({ description: "Task number (required for task_complete, task_remove, task_edit)" })),
    task_file: Type.Optional(Type.String({ description: "File path for the task" })),
    task_verification: Type.Optional(Type.String({ description: "Verification command for the task" })),
    task_notes: Type.Optional(Type.String({ description: "Extra notes for the task" })),
    task_json: Type.Optional(Type.String({ description: "JSON array of tasks (required for task_set)" })),
    ```
  - In `executeTndmCli`, add cases for each new action:
    - `task_add`: `tndm ticket task add <id> --title <title> [--file ...] [--verification ...] [--notes ...] --json`
    - `task_list`: `tndm ticket task list <id> --json`
    - `task_complete`: `tndm ticket task complete <id> <number> --json`
    - `task_remove`: `tndm ticket task remove <id> <number> --json`
    - `task_edit`: `tndm ticket task edit <id> <number> [--title ...] [--file ...] [--verification ...] [--notes ...] --json`
    - `task_set`: `tndm ticket task set <id> --tasks '<json>' --json`
  - Update tool `description` to list new actions
  - Update tests in `__tests__/cli.test.ts` (add integration test entries for task actions)
  - Verification: `cd plugins/supi-flow && pnpm exec tsc --noEmit && pnpm exec vitest run __tests__/cli.test.ts`

- [x] **Task 8**: Rewrite `executeFlowPlan` to use structured tasks
  - File: `plugins/supi-flow/extensions/tools/flow-tools.ts`
  - Remove `doc create plan` + `writeFileSync` + `sync` calls
  - Parse `params.plan_content` markdown to extract tasks:
    - Match lines with regex `/- \[[ x]\] \*\*Task (\d+)\*\*:/` (supports both unchecked and checked tasks in existing plans)
    - For each task line, capture the full title (text after `**Task N**:` up to the end of that line)
    - Build a `TaskInput[]` array: `{ number: N, title: "Task title", status: "todo", file?: string, verification?: string }`
    - Optionally, parse sub-lines (`- File:`, `- Verification:`) following each task header within the same markdown block
  - Call `tndm(["ticket", "task", "set", params.ticket_id, "--tasks", JSON.stringify(tasks)])` via `tndmJson`
  - Update tags: remove all flow-state tags, add `flow:planned` (unchanged logic, keep it)
  - Remove the `append` parameter — no longer needed since task set is a full replace
  - Update `supiFlowPlanParams` schema to remove `append`
  - Verification: `cd plugins/supi-flow && pnpm exec tsc --noEmit && pnpm exec vitest run __tests__/flow-tools.test.ts`

- [x] **Task 9**: Rewrite `executeFlowCompleteTask` to use CLI
  - File: `plugins/supi-flow/extensions/tools/flow-tools.ts`
  - Remove all file-reading/checkTask regex logic
  - Remove imports of `readFileSync`, `writeFileSync`, `dirname`, `join`
  - Call `tndm(["ticket", "task", "complete", params.ticket_id, String(params.task_number), "--json"])` via `tndmJson`
  - Handle errors: if `tndm` throws (task not found / already done), translate to the appropriate error message
  - Keep the soft-fail semantics: if task already completed, `tndm task complete` should either succeed idempotently or return a recognizable error that we translate
  - Verification: `cd plugins/supi-flow && pnpm exec tsc --noEmit && pnpm exec vitest run __tests__/flow-tools.test.ts`

- [x] **Task 10**: Update tool descriptions in `extensions/index.ts`
  - File: `plugins/supi-flow/extensions/index.ts`
  - Update `supi_flow_plan` description: replace "plan.md" references with "structured tasks in state.toml"
  - Update `supi_flow_complete_task` description: replace "plan.md" references with "structured task via tndm ticket task complete"
  - Update `promptGuidelines` for both tools to reflect the task-based approach
  - Update `supi_tndm_cli` description to list the new task actions
  - Verification: `cd plugins/supi-flow && pnpm exec tsc --noEmit`

- [x] **Task 11**: Update plugin tests for new flow tool behavior
  - File: `plugins/supi-flow/__tests__/flow-tools.test.ts`
  - Update `executeFlowPlan` tests:
    - Mock `tndm`/`tndmJson` to return task set results instead of doc create
    - Verify `tndm` is called with `["ticket", "task", "set", "TNDM-TEST", "--tasks", ...]`
    - Verify no `doc create` call
    - Verify markdown parsing produces correct task objects
    - Test markdown parsing: task with file/verification sub-lines, multiple tasks, checked tasks in existing plans
  - Update `executeFlowCompleteTask` tests:
    - Mock `tndmJson` to return success/error for `task complete`
    - Remove all filesystem-based tests (no more `writeFileSync`/`readFileSync` mocks needed)
    - Test success path: `task complete` returns JSON with completed task
    - Test not-found path: `task complete` throws with appropriate error message
    - Test already-done path: if we make `task complete` idempotent, verify soft-fail behavior
  - Update `executeFlowClose` tests — no changes expected (close still uses archive)
  - Verification: `cd plugins/supi-flow && pnpm exec vitest run __tests__/flow-tools.test.ts`

- [x] **Task 12**: Remove `plan.md` references from supi-flow skills
  - Files:
    - `plugins/supi-flow/skills/supi-flow-plan/SKILL.md`
    - `plugins/supi-flow/skills/supi-flow-apply/SKILL.md`
    - `plugins/supi-flow/skills/supi-flow-archive/SKILL.md`
  - Changes:
    - Replace "plan.md" with "tasks in state.toml" or "structured tasks"
    - Update `supi_flow_plan` parameter descriptions: `plan_content` is now parsed into tasks (the markdown format is still how the agent writes it, but storage is task-based)
    - In `supi-flow-apply`, replace "read plan.md" with "read tasks via `supi_tndm_cli { action: "task_list", id: "<ID>" }`"
    - In `supi-flow-archive`, replace references to reading plan.md
    - Update `supi-flow-plan` Output and persistence section: reference structured tasks instead of plan.md file
  - Verification: manual review — read each modified skill file and confirm no plan.md references remain
  - **Test-exempt** (docs-only changes; verify by reading files)

- [x] **Task 13**: Run full verification suite
  - Run all Rust tests: `cargo test --workspace`
  - Run all TypeScript tests: `cd plugins/supi-flow && pnpm exec vitest run`
  - Run architecture check: `cargo xtask check-arch`
  - Run CLI format check: `./tndm-dev fmt --check`
  - Verification: all commands exit 0, no failures
