# Structured Task Management in TNDM

## Problem

Today tasks are free-form markdown in `plan.md`, manipulated with brittle regex (`checkTask` parses `^- \[ \] \*\*Task N\*\*:`). There is no structured schema, no CLI query capability, and no reliable diff/awareness for task changes. The supi-flow plugin regex-searches markdown, string-replaces `[ ]` → `[x]`, writes it back, and calls sync. This is fragile and error-prone.

## Solution

Store tasks as structured data in `state.toml`, remove `plan.md`, and add a `tndm ticket task` CLI subcommand.

### Data model

```rust
pub enum TaskStatus { Todo, Done }

pub struct Task {
    pub number: u32,
    pub title: String,
    pub status: TaskStatus,
    pub file: Option<String>,       // affected file path
    pub verification: Option<String>, // verification command
    pub notes: Option<String>,      // extra description
}
```

Added to `TicketState`:
```rust
pub struct TicketState {
    pub status: TicketStatus,
    pub updated_at: String,
    pub revision: u64,
    pub document_fingerprints: BTreeMap<String, String>,
    pub tasks: Vec<Task>,           // NEW
}
```

### CLI surface

```
tndm ticket task add <id> --title "..." [--file path] [--verification "cmd"] [--notes "..."]
tndm ticket task list <id> [--json]
tndm ticket task complete <id> <number>
tndm ticket task remove <id> <number>
tndm ticket task edit <id> <number> [--title ...] [--file ...] [--verification ...] [--notes ...]
```

### supi-flow changes

| Tool | Current behavior | New behavior |
|---|---|---|
| `supi_flow_plan` | Writes markdown to `plan.md` | Parses markdown plan into structured tasks, calls `tndm ticket task set <id> --json <tasks>` (bulk replace). No `plan` document. |
| `supi_flow_complete_task` | Regex-replaces `[ ]` → `[x]` in `plan.md` | Calls `tndm ticket task complete <id> <number>` |
| `supi_flow_close` | Creates `archive.md` | Unchanged |
| `supi_flow_start` | Creates ticket with `content.md` | Unchanged |

### Backwards compatibility

None. `plan.md` is dead for new tickets. Existing tickets with `plan.md` are left untouched. supi-flow tools operate on structured tasks only.

### Files to touch

**Rust (tndm core + CLI + storage):**
- `crates/tandem-core/src/ticket.rs` — `Task`, `TaskStatus`, update `TicketState`
- `crates/tandem-storage/src/lib.rs` — serialize/deserialize `tasks` in state
- `crates/tandem-cli/src/main.rs` — `TicketTaskCommand` subcommands, JSON output

**TypeScript (supi-flow plugin):**
- `plugins/supi-flow/extensions/tools/flow-tools.ts` — rewrite `executeFlowPlan` and `executeFlowCompleteTask`
- `plugins/supi-flow/extensions/tools/tndm-cli.ts` — add task actions to `supi_tndm_cli`
- `plugins/supi-flow/extensions/index.ts` — update tool descriptions
- `plugins/supi-flow/skills/*` — remove `plan.md` references, update task conventions
- `plugins/supi-flow/__tests__/flow-tools.test.ts` — update tests

## Constraints / non-goals

- No migration of existing plan.md files
- No subtasks / nested tasks (flat list only)
- No plan.md auto-generation from tasks (tasks are the single source of truth)
