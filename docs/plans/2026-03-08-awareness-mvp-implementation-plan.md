# Awareness MVP Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement `tndm awareness --against <ref>` so it compares all tickets in the current checkout against one Git ref and emits deterministic JSON while preserving the repository’s enforced architecture.

**Architecture:** Keep `tandem-core` IO-free by putting the awareness report types and snapshot comparison logic there. Keep `tandem-repo` focused on Git/ref access only: it should resolve refs, list tracked ticket files, and materialize ref snapshots, but it must not depend on `tandem-storage`. Keep `tandem-cli` as the orchestration layer: load the current snapshot via `tandem-storage`, ask `tandem-repo` to materialize the target ref snapshot, parse that snapshot via `tandem-storage`, compare via `tandem-core`, and print deterministic JSON.

**Tech Stack:** Rust workspace, `serde` + `serde_json`, `tempfile`, `std::process::Command` for git access in `tandem-repo`, `clap` in `tandem-cli`, Rust test harness.

---

## Pre-flight notes (read before starting)

- Product intent and output contract live in `docs/plans/2026-03-07-awareness-mvp-design.md`.
- Workspace boundaries are enforced by `docs/architecture.md` and `cargo xtask check-arch`.
- `tandem-core` must remain IO-free.
- `tandem-repo` may depend on `tandem-core` only among workspace crates.
- `tandem-storage` is the only crate that should parse/load ticket files.
- `tandem-cli` may depend on `tandem-core`, `tandem-storage`, and `tandem-repo`, so it is the correct place to orchestrate awareness.
- Keep the MVP narrow:
  - compare one target ref only
  - JSON output only
  - tracked fields: `status`, `priority`, `depends_on`
  - no `content.md` diffing
  - no semantic conflict categories

## Output contract (to test against)

### `tndm awareness --against <ref>`

Print pretty JSON with this top-level shape:

```json
{
  "schema_version": 1,
  "against": "main",
  "tickets": []
}
```

For changed tickets, each entry must have:

```json
{
  "id": "TNDM-AAAAAA",
  "change": "diverged",
  "fields": {
    "status": {
      "current": "in_progress",
      "against": "todo"
    },
    "priority": {
      "current": "p1",
      "against": "p2"
    },
    "depends_on": {
      "current": ["TNDM-000001"],
      "against": []
    }
  }
}
```

Rules:

- top-level keys appear in this order: `schema_version`, `against`, `tickets`
- `tickets` are sorted by ticket ID
- `fields` keys appear in this order when present: `status`, `priority`, `depends_on`
- unchanged fields are omitted
- change kinds are exactly:
  - `added_current`
  - `added_against`
  - `diverged`

---

## Task 0: Create an isolated worktree + feature branch

**Files:**
- (none)

**Step 1: Create worktree + branch**

Use @superpowers:using-git-worktrees or run an equivalent git worktree command.

Example:

```bash
git worktree add .claude/worktrees/awareness-mvp -b feat/awareness-mvp
```

Expected: new worktree exists on branch `feat/awareness-mvp`.

**Step 2: Verify clean state**

Run:

```bash
git -C .claude/worktrees/awareness-mvp status --short
```

Expected: no output.

**Step 3: Commit**

- No commit in this task.

---

## Task 1: Add core awareness types and pure snapshot comparison logic

**Files:**
- Modify: `crates/tandem-core/Cargo.toml`
- Modify: `crates/tandem-core/src/lib.rs`
- Create: `crates/tandem-core/src/awareness.rs`

**Step 1: Write the failing tests**

Create `crates/tandem-core/src/awareness.rs` with tests first. Add tests for:

- identical snapshots produce an empty report
- current-only tickets become `added_current`
- target-only tickets become `added_against`
- differing `status`, `priority`, and `depends_on` become `diverged`
- output ordering is stable by ticket ID and field key order
- unchanged fields are omitted from serialized JSON
- identical `depends_on` values in different orders do not diverge
- duplicate `depends_on` values remain visible as differences

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p tandem-core --locked
```

Expected: FAIL because the awareness module/types/functions do not exist yet.

**Step 3: Write minimal implementation**

In `crates/tandem-core/Cargo.toml`, add:

```toml
[dependencies]
serde.workspace = true
time.workspace = true

[dev-dependencies]
serde_json.workspace = true
```

In `crates/tandem-core/src/lib.rs`, export the new module:

```rust
pub mod awareness;
```

In `crates/tandem-core/src/awareness.rs`, implement:

- `TicketSnapshot { tickets: BTreeMap<TicketId, Ticket> }`
- `impl TicketSnapshot { pub fn from_tickets(...) -> Self }`
- `AwarenessReport`
- `AwarenessTicketChange`
- `AwarenessChangeKind`
- field diff structs with fixed field order:
  - `status`
  - `priority`
  - `depends_on`
- `compare_snapshots(against: impl Into<String>, current: &TicketSnapshot, against_snapshot: &TicketSnapshot) -> AwarenessReport`

Implementation rules:

- use `BTreeMap` and `BTreeSet` so ticket ordering is stable
- report `id` as `String` in JSON-facing types
- compare only:
  - `ticket.state.status.as_str()`
  - `ticket.meta.priority.as_str()`
  - `ticket.meta.depends_on` as normalized string arrays
- ignore `content` and `updated_at`
- omit unchanged tickets entirely
- omit unchanged fields from `diverged` entries
- derive `serde::Serialize` on report/output structs

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-core --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml crates/tandem-core/Cargo.toml crates/tandem-core/src/lib.rs crates/tandem-core/src/awareness.rs Cargo.lock
git commit -m "feat(core): add awareness snapshot comparison"
```

---

## Task 2: Add a storage helper that loads all tickets into a snapshot

**Files:**
- Modify: `crates/tandem-storage/src/lib.rs`
- Test: `crates/tandem-storage/tests/awareness_storage_tests.rs`

**Step 1: Write the failing tests**

Create `crates/tandem-storage/tests/awareness_storage_tests.rs` with tests for:

- `load_ticket_snapshot_returns_sorted_tickets()`
- `load_ticket_snapshot_returns_empty_when_tickets_dir_is_missing()`

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: FAIL because `load_ticket_snapshot` does not exist yet.

**Step 3: Write minimal implementation**

In `crates/tandem-storage/src/lib.rs`, add:

```rust
use std::collections::BTreeMap;
use tandem_core::awareness::TicketSnapshot;
```

Then implement:

```rust
pub fn load_ticket_snapshot(repo_root: &Path) -> Result<TicketSnapshot, StorageError> {
    let store = FileTicketStore::new(repo_root.to_path_buf());
    let mut tickets = BTreeMap::new();

    for id in store.list_ticket_ids()? {
        let ticket = store.load_ticket(&id)?;
        tickets.insert(id, ticket);
    }

    Ok(TicketSnapshot { tickets })
}
```

Implementation notes:

- this helper should return an empty snapshot if `.tndm/tickets` is absent
- do not add Git logic here
- reuse existing `FileTicketStore` methods rather than duplicating read/parse code

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/tandem-storage/src/lib.rs crates/tandem-storage/tests/awareness_storage_tests.rs
git commit -m "feat(storage): add ticket snapshot loader"
```

---

## Task 3: Replace the legacy awareness port and implement Git/ref materialization only

**Files:**
- Modify: `crates/tandem-core/src/ports.rs`
- Modify: `crates/tandem-repo/Cargo.toml`
- Modify: `crates/tandem-repo/src/lib.rs`
- Test: `crates/tandem-repo/tests/awareness_repo_tests.rs`

**Step 1: Write the failing tests**

Create `crates/tandem-repo/tests/awareness_repo_tests.rs` with tests for:

- `materialize_ref_snapshot_writes_committed_ticket_files()`
- `materialize_ref_snapshot_returns_empty_when_ref_has_no_tickets()`
- `materialize_ref_snapshot_errors_for_unknown_ref()`
- `materialize_ref_snapshot_sanitizes_temp_paths_for_invalid_committed_ticket_data()`

Test through the repo-layer API only. Do not depend on `tandem-storage` in production code to satisfy the test.

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p tandem-repo --locked
```

Expected: FAIL because the new port/API and git ref materialization do not exist yet.

**Step 3: Write minimal implementation**

In `crates/tandem-core/src/ports.rs`, replace the old single-ticket awareness abstraction with a Git-materialization-oriented port:

```rust
use std::path::PathBuf;

pub trait AwarenessRefMaterializer {
    type Error;

    fn materialize_ref_snapshot(&self, reference: &str) -> Result<Option<PathBuf>, Self::Error>;
}
```

Keep `RepoContext` if still needed. Remove the old `TicketChange` struct and the old `AwarenessProvider` trait if they are no longer used.

In `crates/tandem-repo/Cargo.toml`, add:

```toml
[dependencies]
tandem-core.workspace = true
tempfile.workspace = true
```

Do **not** add `tandem-storage`.

In `crates/tandem-repo/src/lib.rs`, implement:

- `GitAwarenessProvider { repo_root: PathBuf }`
- `impl GitAwarenessProvider { pub fn new(repo_root: PathBuf) -> Self }`
- `impl AwarenessRefMaterializer for GitAwarenessProvider`

Implementation outline:

1. `materialize_ref_snapshot(reference)`
   - verify the ref resolves with:

   ```bash
   git rev-parse --verify <ref>^{commit}
   ```

   - list tracked ticket files at that ref with:

   ```bash
   git ls-tree -r --name-only <ref> -- .tndm/tickets
   ```

   - if no paths are returned, return `Ok(None)`
   - create a temp root via `tempfile::tempdir()`
   - persist the tempdir so the returned path remains valid after function return
   - for each returned file path, fetch its blob with:

   ```bash
   git show <ref>:<path>
   ```

   - write the blob into the temp root using the same relative path
   - return `Ok(Some(path_to_materialized_root))`

2. Add internal helpers:
   - `run_git(repo_root: &Path, args: &[&str]) -> Result<Vec<u8>, RepoError>`
   - `list_ref_ticket_paths(...)`
   - `write_ref_ticket_tree(...)`

3. Error handling
   - include the failing git command or ref name in errors
   - do not include absolute repo-root or tempfile paths in deterministic error messages
   - sanitize materialized-root path text to `<ref-snapshot>` where needed

Implementation note:

Because the function returns a `PathBuf`, you need a way to keep the temp directory alive after return. Use one of these minimal approaches:

- store a managed tempdir handle inside `GitAwarenessProvider` keyed by ref/materialization request, or
- return a dedicated owned handle type instead of raw `PathBuf` if you revise the port signature accordingly.

If you revise the port signature, keep the abstraction in `tandem-core` and keep `tandem-repo` free of `tandem-storage`.

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-repo --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/tandem-core/src/ports.rs crates/tandem-repo/Cargo.toml crates/tandem-repo/src/lib.rs crates/tandem-repo/tests/awareness_repo_tests.rs Cargo.lock
git commit -m "feat(repo): materialize awareness snapshots from git refs"
```

---

## Task 4: Implement `tndm awareness --against <ref>` with CLI-level orchestration and deterministic JSON output

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/tandem-cli/Cargo.toml`
- Modify: `crates/tandem-cli/src/main.rs`
- Test: `crates/tandem-cli/tests/awareness_cli_tests.rs`

**Step 1: Write the failing CLI tests**

Create `crates/tandem-cli/tests/awareness_cli_tests.rs` with tests for:

- `awareness_prints_empty_json_when_snapshots_match()`
- `awareness_reports_added_current_added_against_and_diverged_sorted()`
- `awareness_errors_for_invalid_ref()`
- `awareness_errors_for_invalid_committed_ticket_data_without_temp_path_leakage()`

Assert exact pretty JSON for the success cases.

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p tandem-cli --locked
```

Expected: FAIL because the CLI flag/handler and orchestration do not exist yet.

**Step 3: Write minimal implementation**

In root `Cargo.toml`, ensure:

```toml
[workspace.dependencies]
serde_json = "1"
```

In `crates/tandem-cli/Cargo.toml`, add:

```toml
[dependencies]
serde_json.workspace = true
```

In `crates/tandem-cli/src/main.rs`:

1. Update the command definition:

```rust
/// Show awareness of relevant ticket changes elsewhere.
Awareness {
    #[arg(long)]
    against: String,
},
```

2. Add imports:

```rust
use tandem_core::{
    awareness::compare_snapshots,
    ports::AwarenessRefMaterializer,
};
use tandem_repo::GitAwarenessProvider;
use tandem_storage::load_ticket_snapshot;
```

3. Replace the stub with a real handler:

```rust
Command::Awareness { against } => handle_awareness(against),
```

4. Implement `handle_awareness(against: String) -> anyhow::Result<()>`:

- discover repo root from current dir
- load current snapshot with `tandem_storage::load_ticket_snapshot(&repo_root)`
- construct `GitAwarenessProvider::new(repo_root.clone())`
- materialize the target ref snapshot via repo layer
- if materializer returns `None`, use `TicketSnapshot::default()` for the target side
- if it returns a snapshot root path/handle, parse it with `tandem_storage::load_ticket_snapshot(...)`
- if that parse fails, wrap the error in a deterministic CLI error message that includes the ref and sanitized `<ref-snapshot>` path context rather than a machine-specific temp path
- call `compare_snapshots(&against, &current, &against_snapshot)`
- render with `serde_json::to_string_pretty(&report)`
- print the JSON followed by a newline

Minimal implementation sketch:

```rust
fn handle_awareness(against: String) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;

    let current = load_ticket_snapshot(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;
    let provider = GitAwarenessProvider::new(repo_root.clone());

    let against_snapshot = match provider
        .materialize_ref_snapshot(&against)
        .map_err(|error| anyhow::anyhow!("{error}"))?
    {
        None => tandem_core::awareness::TicketSnapshot::default(),
        Some(snapshot_root) => load_ticket_snapshot(&snapshot_root).map_err(|error| {
            anyhow::anyhow!(
                "failed to load materialized snapshot for ref `{}`: {}",
                against,
                error.to_string().replace(snapshot_root.to_string_lossy().as_ref(), "<ref-snapshot>")
            )
        })?,
    };

    let report = compare_snapshots(&against, &current, &against_snapshot);
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
```

Keep the CLI thin: orchestration only, no direct Git command execution.

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-cli --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml crates/tandem-cli/Cargo.toml crates/tandem-cli/src/main.rs crates/tandem-cli/tests/awareness_cli_tests.rs Cargo.lock
git commit -m "feat(cli): implement awareness summary command"
```

---

## Task 5: Full workspace verification

**Files:**
- (none)

**Step 1: Run targeted tests**

Run:

```bash
cargo test -p tandem-core --locked
cargo test -p tandem-storage --locked
cargo test -p tandem-repo --locked
cargo test -p tandem-cli --locked
```

Expected: PASS.

**Step 2: Run architecture check first**

Run:

```bash
cargo xtask check-arch
```

Expected: PASS.

This check is called out separately because awareness work crosses multiple crates and must not introduce forbidden dependency edges.

**Step 3: Run full repo checks from the repository root**

Run:

```bash
cd "$(git rev-parse --show-toplevel)" && mise run check
```

Expected: PASS (fmt, compile, arch, clippy, test).

**Step 4: Commit**

- No commit unless verification requires follow-up fixes.

---

## Execution handoff

Plan complete and saved to `docs/plans/2026-03-08-awareness-mvp-implementation-plan.md`.

Two execution options:

1. **Subagent-Driven (this session)** — I dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Parallel Session (separate)** — Open a new session in a worktree and run tasks sequentially with checkpoints using `superpowers:executing-plans`

Which approach do you want?
