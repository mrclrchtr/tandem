# Ticket commands (create/show/list) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement V1 `tndm ticket create/show/list` backed by repo-local, deterministic files under `.tndm/tickets/<ID>/`.

**Architecture:** Keep `tandem-core` IO-free and focused on domain types + validation + canonical formatting. Implement filesystem IO and TOML parsing in `tandem-storage`. Keep `tandem-cli` thin: parse args, resolve content source precedence (file/stdin/template), generate IDs, call `tandem-storage`.

**Tech Stack:** Rust (workspace, edition 2024), `clap` (CLI only), `serde` + `toml` (parsing config/meta/state), `rand` (ID suffix generation), `time` (RFC3339 timestamps), `tempfile` + `assert_cmd` + `predicates` (tests).

---

## Pre-flight notes (read before starting)

- Repo architecture constraints are documented in `docs/architecture.md`.
- Only `crates/tandem-cli` may depend on `clap`.
- `crates/tandem-core` must remain IO-free (see `clippy.toml` disallowed fs/process APIs).
- Determinism: writer paths should emit canonical TOML without relying on “pretty printer” formatting stability.
- Use small commits matching existing style (e.g. `feat(storage): ...`, `feat(cli): ...`, `test(storage): ...`, `docs(plans): ...`).

## Output contracts (to test against)

### `tndm ticket create`

- Prints the created ticket ID to stdout as the only output (plus newline).

### `tndm ticket show <id>`

- Prints three sections (deterministic headers) in this order:

```
## meta.toml
<canonical meta.toml>

## state.toml
<canonical state.toml>

## content.md
<raw content.md>
```

### `tndm ticket list`

- Prints one line per ticket, sorted by ticket ID string.
- Line format (tab-separated):

```
<ID>\t<status>\t<title>
```

Example:

```
TNDM-4K7D9Q\ttodo\tAdd foo
```

---

## Task 0: Create an isolated worktree + feature branch

**Files:**
- (none)

**Step 1: Create worktree + branch**

Run (example):

```bash
git worktree add .claude/worktrees/ticket-commands -b feat/ticket-commands
```

Expected: new worktree directory exists and is on branch `feat/ticket-commands`.

**Step 2: Verify clean state**

Run:

```bash
git status
```

Expected: working tree clean.

**Step 3: Commit**

- No commit in this task.

---

## Task 1: Add dependencies for storage parsing, ID generation, timestamps, and tests

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/tandem-storage/Cargo.toml`
- Modify: `crates/tandem-cli/Cargo.toml`

**Step 1: Write a failing test (dependency-driven)**

- Add a minimal test placeholder that uses `tempfile` in storage tests (will not compile until deps are added).
- Create: `crates/tandem-storage/src/lib.rs` (test module at bottom) or `crates/tandem-storage/tests/config_tests.rs`.

Example (new file `crates/tandem-storage/tests/config_tests.rs`):

```rust
#[test]
fn placeholder_uses_tempfile() {
    let _dir = tempfile::tempdir().expect("tempdir");
}
```

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: compile error complaining `tempfile` not found.

**Step 3: Add minimal dependencies**

1) In root `Cargo.toml`, add to `[workspace.dependencies]`:

```toml
rand = "0.8"
serde = { version = "1", features = ["derive"] }
time = { version = "0.3", features = ["formatting", "parsing"] }
toml = "0.8"

assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

2) In `crates/tandem-storage/Cargo.toml`:

```toml
[dependencies]
serde.workspace = true
time.workspace = true
toml.workspace = true
tandem-core.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

3) In `crates/tandem-cli/Cargo.toml`:

```toml
[dependencies]
anyhow.workspace = true
clap.workspace = true
rand.workspace = true

tandem-core.workspace = true
tandem-repo.workspace = true

tandem-storage.workspace = true

[dev-dependencies]
assert_cmd.workspace = true
predicates.workspace = true
tempfile.workspace = true
```

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml crates/tandem-storage/Cargo.toml crates/tandem-cli/Cargo.toml crates/tandem-storage/tests/config_tests.rs
git commit -m "chore(deps): add toml/serde/rand/time test deps"
```

---

## Task 2: Extend core domain model for ticket meta/state and canonical TOML formatting

**Files:**
- Modify: `crates/tandem-core/src/ticket.rs`

**Step 1: Write the failing tests**

Append tests to `crates/tandem-core/src/ticket.rs` verifying:

- parsing `TicketType`, `TicketPriority`, `TicketStatus`
- canonical TOML formatting for `TicketMeta` and `TicketState` is stable

Example test skeleton:

```rust
#[test]
fn meta_formats_as_canonical_toml() {
    let id = TicketId::parse("TNDM-4K7D9Q").unwrap();
    let meta = TicketMeta::new(id, "Add foo").unwrap();

    assert_eq!(
        meta.to_canonical_toml(),
        concat!(
            "schema_version = 1\n",
            "id = \"TNDM-4K7D9Q\"\n",
            "title = \"Add foo\"\n",
            "\n",
            "type = \"task\"\n",
            "priority = \"p2\"\n",
            "\n",
            "depends_on = []\n",
            "tags = []\n",
        )
    );
}
```

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p tandem-core --locked
```

Expected: compile errors (types/functions don’t exist yet).

**Step 3: Implement minimal core types + formatting**

In `crates/tandem-core/src/ticket.rs`, add:

- `TicketType` enum with `as_str()` and `parse()`.
- `TicketPriority` enum with `as_str()` and `parse()`.
- `TicketStatus` enum with `as_str()` and `parse()`.
- Extend `TicketId` to derive `PartialOrd`/`Ord` so it can be sorted.
- `TicketMeta` struct with defaults:
  - `type = task`
  - `priority = p2`
  - `depends_on = []`
  - `tags = []`
- `TicketState` struct with:
  - `status` (default `todo`)
  - `updated_at` (string)
  - `revision` (must be >= 1)
- `to_canonical_toml()` methods that write TOML with fixed key order.

Minimal implementation sketch (partial):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TicketStatus {
    Todo,
    InProgress,
    Blocked,
    Done,
}

impl TicketStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Done => "done",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ValidationError> {
        match value {
            "todo" => Ok(Self::Todo),
            "in_progress" => Ok(Self::InProgress),
            "blocked" => Ok(Self::Blocked),
            "done" => Ok(Self::Done),
            _ => Err(ValidationError::new("invalid ticket status")),
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-core --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/tandem-core/src/ticket.rs
git commit -m "feat(core): add ticket meta/state types and canonical TOML"
```

---

## Task 3: Update core TicketStore port to support create/load/list

**Files:**
- Modify: `crates/tandem-core/src/ports.rs`

**Step 1: Write the failing test (compile-driven)**

Add a small “uses new trait” test in storage that will fail until the trait exists.

Edit `crates/tandem-storage/tests/config_tests.rs` to reference the new API (temporary compile guard):

```rust
#[test]
fn placeholder_ticketstore_api_exists() {
    // This test is only to force compilation of the updated trait API.
    let _ = std::mem::size_of::<tandem_core::ticket::TicketMeta>();
}
```

**Step 2: Run tests to verify it fails**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: compile errors if symbols don’t exist yet.

**Step 3: Implement the updated port trait**

In `crates/tandem-core/src/ports.rs`, change `TicketStore` to:

```rust
use crate::ticket::{NewTicket, Ticket, TicketId};

pub trait TicketStore {
    type Error;

    fn create_ticket(&self, ticket: NewTicket) -> Result<Ticket, Self::Error>;
    fn load_ticket(&self, id: &TicketId) -> Result<Ticket, Self::Error>;
    fn list_ticket_ids(&self) -> Result<Vec<TicketId>, Self::Error>;
    fn ticket_exists(&self, id: &TicketId) -> Result<bool, Self::Error>;
}
```

(Define `NewTicket` and `Ticket` in `crates/tandem-core/src/ticket.rs` as simple structs.)

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-core --locked
cargo test -p tandem-storage --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/tandem-core/src/ports.rs crates/tandem-core/src/ticket.rs crates/tandem-storage/tests/config_tests.rs
git commit -m "feat(core): expand TicketStore port for create/load/list"
```

---

## Task 4: Implement `.tndm/config.toml` loading in tandem-storage

**Files:**
- Modify: `crates/tandem-storage/src/lib.rs` (or split into modules if it grows)
- Test: `crates/tandem-storage/tests/config_tests.rs`

**Step 1: Write the failing tests**

In `crates/tandem-storage/tests/config_tests.rs`, add:

- `load_config_defaults_when_missing()`
- `load_config_reads_prefix_and_template()`
- `load_config_rejects_unknown_schema_version()`

Example:

```rust
#[test]
fn load_config_reads_prefix_and_template() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".tndm")).unwrap();

    std::fs::write(
        dir.path().join(".tndm/config.toml"),
        "schema_version = 1\n\n[id]\nprefix = \"FOO\"\n\n[templates]\ncontent = '''\nHello\n'''\n",
    )
    .unwrap();

    let cfg = tandem_storage::load_config(dir.path()).unwrap();
    assert_eq!(cfg.id_prefix, "FOO");
    assert_eq!(cfg.content_template, "Hello\n");
}
```

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: FAIL (missing `load_config`).

**Step 3: Implement minimal config loader**

In `crates/tandem-storage/src/lib.rs` add:

- `pub struct TandemConfig { pub id_prefix: String, pub content_template: String }`
- `pub const DEFAULT_ID_PREFIX: &str = "TNDM";`
- `pub const DEFAULT_CONTENT_TEMPLATE: &str = "## Description\n\n## Design\n\n## Acceptance\n\n## Notes\n";`
- `pub fn load_config(repo_root: &Path) -> Result<TandemConfig, StorageError>`

Implementation notes:

- Read `.tndm/config.toml` if it exists, else return defaults.
- Parse via `toml::from_str` into a deserialization struct (using `serde::Deserialize`).
- If `schema_version` exists and is not `1`, return an error.

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/tandem-storage/src/lib.rs crates/tandem-storage/tests/config_tests.rs
git commit -m "feat(storage): load .tndm config for id prefix and templates"
```

---

## Task 5: Implement FileTicketStore paths + repo root discovery

**Files:**
- Modify: `crates/tandem-storage/src/lib.rs`
- Test: `crates/tandem-storage/tests/ticket_store_tests.rs`

**Step 1: Write the failing tests**

Create `crates/tandem-storage/tests/ticket_store_tests.rs`:

- `discover_repo_root_finds_git_dir()`
- `discover_repo_root_errors_when_no_repo_markers()`

Example:

```rust
#[test]
fn discover_repo_root_finds_git_dir() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let nested = dir.path().join("a/b/c");
    std::fs::create_dir_all(&nested).unwrap();

    let root = tandem_storage::discover_repo_root(&nested).unwrap();
    assert_eq!(root, dir.path());
}
```

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: FAIL (missing `discover_repo_root`).

**Step 3: Implement minimal discovery + path helpers**

In `crates/tandem-storage/src/lib.rs` implement:

- `pub fn discover_repo_root(start: &Path) -> Result<PathBuf, StorageError>`
  - ascend parents looking for `.tndm/` OR `.git/` (dir or file)
  - if not found, return an error indicating that no repository markers were found

And a `FileTicketStore` that stores `repo_root: PathBuf` and computes:

- `.tndm/`
- `.tndm/tickets/`
- `.tndm/tickets/<id>/`

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/tandem-storage/src/lib.rs crates/tandem-storage/tests/ticket_store_tests.rs
git commit -m "feat(storage): add repo root discovery and ticket path helpers"
```

---

## Task 6: Implement `FileTicketStore::create_ticket` (writes meta/state/content)

**Files:**
- Modify: `crates/tandem-storage/src/lib.rs`
- Test: `crates/tandem-storage/tests/ticket_store_tests.rs`

**Step 1: Write the failing test**

Add to `ticket_store_tests.rs`:

```rust
#[test]
fn create_ticket_writes_expected_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let store = tandem_storage::FileTicketStore::new(dir.path().to_path_buf());

    let id = tandem_core::ticket::TicketId::parse("TNDM-4K7D9Q").unwrap();
    let meta = tandem_core::ticket::TicketMeta::new(id.clone(), "Add foo").unwrap();

    let created = store
        .create_ticket(tandem_core::ticket::NewTicket {
            meta,
            content: "Hello\n".to_string(),
        })
        .unwrap();

    assert_eq!(created.meta.id.as_str(), "TNDM-4K7D9Q");

    let base = dir
        .path()
        .join(".tndm/tickets/TNDM-4K7D9Q");

    assert!(base.join("meta.toml").exists());
    assert!(base.join("state.toml").exists());
    assert!(base.join("content.md").exists());
}
```

**Step 2: Run tests to verify it fails**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: FAIL (not implemented).

**Step 3: Implement minimal create**

In `crates/tandem-storage/src/lib.rs`:

- Implement `TicketStore for FileTicketStore` using the updated trait.
- `create_ticket` should:
  - `create_dir_all(.tndm/tickets)`
  - `create_dir(ticket_dir)` (error if exists)
  - write `meta.toml` using `ticket.meta.to_canonical_toml()`
  - create `TicketState` with:
    - status `todo`
    - `updated_at = now_utc RFC3339` (use `time`)
    - `revision = 1`
  - write `state.toml` using `state.to_canonical_toml()`
  - write `content.md` as provided

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/tandem-storage/src/lib.rs crates/tandem-storage/tests/ticket_store_tests.rs
git commit -m "feat(storage): implement FileTicketStore create_ticket"
```

---

## Task 7: Implement `FileTicketStore::load_ticket` (parses meta/state + reads content)

**Files:**
- Modify: `crates/tandem-storage/src/lib.rs`
- Test: `crates/tandem-storage/tests/ticket_store_tests.rs`

**Step 1: Write the failing test**

Add:

```rust
#[test]
fn load_ticket_roundtrips_created_ticket() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let store = tandem_storage::FileTicketStore::new(dir.path().to_path_buf());

    let id = tandem_core::ticket::TicketId::parse("TNDM-4K7D9Q").unwrap();
    let meta = tandem_core::ticket::TicketMeta::new(id.clone(), "Add foo").unwrap();

    let _ = store
        .create_ticket(tandem_core::ticket::NewTicket {
            meta,
            content: "Hello\n".to_string(),
        })
        .unwrap();

    let loaded = store.load_ticket(&id).unwrap();
    assert_eq!(loaded.meta.title, "Add foo");
    assert_eq!(loaded.state.status.as_str(), "todo");
    assert_eq!(loaded.content, "Hello\n");
}
```

**Step 2: Run tests to verify it fails**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: FAIL.

**Step 3: Implement minimal TOML parsing + validation**

In `crates/tandem-storage/src/lib.rs`:

- Read meta/state TOML as strings.
- Deserialize into internal structs, e.g.:

```rust
#[derive(serde::Deserialize)]
struct MetaFile {
    schema_version: u32,
    id: String,
    title: String,
    #[serde(rename = "type")]
    kind: Option<String>,
    priority: Option<String>,
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}
```

- Validate `schema_version == 1`.
- Parse IDs via `TicketId::parse`.
- Map `kind/priority/status` strings via core `parse()` functions.
- Ensure meta `id` matches directory name.
- Return `Ticket { meta, state, content }`.

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/tandem-storage/src/lib.rs crates/tandem-storage/tests/ticket_store_tests.rs
git commit -m "feat(storage): implement FileTicketStore load_ticket"
```

---

## Task 8: Implement `FileTicketStore::list_ticket_ids`

**Files:**
- Modify: `crates/tandem-storage/src/lib.rs`
- Test: `crates/tandem-storage/tests/ticket_store_tests.rs`

**Step 1: Write the failing test**

Add:

```rust
#[test]
fn list_ticket_ids_sorts_by_id() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let store = tandem_storage::FileTicketStore::new(dir.path().to_path_buf());

    for (id, title) in [
        ("TNDM-BBBBBB", "B"),
        ("TNDM-AAAAAA", "A"),
    ] {
        let id = tandem_core::ticket::TicketId::parse(id).unwrap();
        let meta = tandem_core::ticket::TicketMeta::new(id, title).unwrap();
        let _ = store
            .create_ticket(tandem_core::ticket::NewTicket {
                meta,
                content: "x\n".to_string(),
            })
            .unwrap();
    }

    let ids = store.list_ticket_ids().unwrap();
    assert_eq!(ids[0].as_str(), "TNDM-AAAAAA");
    assert_eq!(ids[1].as_str(), "TNDM-BBBBBB");
}
```

**Step 2: Run tests to verify it fails**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: FAIL.

**Step 3: Implement minimal listing**

- `read_dir(.tndm/tickets)`
- collect directory names
- `TicketId::parse(name)` for each
- sort and return

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-storage --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/tandem-storage/src/lib.rs crates/tandem-storage/tests/ticket_store_tests.rs
git commit -m "feat(storage): implement list_ticket_ids"
```

---

## Task 9: Implement `tndm ticket create` CLI (ID generation + content precedence)

**Files:**
- Modify: `crates/tandem-cli/src/main.rs`
- Test: `crates/tandem-cli/tests/ticket_cli_tests.rs`

**Step 1: Write the failing integration test**

Create `crates/tandem-cli/tests/ticket_cli_tests.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn ticket_create_generates_id_and_writes_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let mut cmd = Command::cargo_bin("tndm").unwrap();
    cmd.current_dir(dir.path())
        .args(["ticket", "create", "Add foo"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("TNDM-")
            .and(predicate::str::is_match(r"^TNDM-[0-9A-Z]{6}\n$").unwrap()));
}
```

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test -p tandem-cli --locked
```

Expected: FAIL (command not implemented).

**Step 3: Implement minimal CLI create**

In `crates/tandem-cli/src/main.rs`:

- Update clap definitions:

```rust
enum TicketCommand {
    Create {
        #[arg(long)]
        id: Option<String>,

        #[arg(long)]
        content_file: Option<std::path::PathBuf>,

        title: String,
    },
    Show { id: String },
    List,
}
```

- Implement `create`:
  - discover repo root: `tandem_storage::discover_repo_root(&std::env::current_dir()?)`
  - load config: `tandem_storage::load_config(&repo_root)`
  - pick ID:
    - if `--id`, use it
    - else generate random suffix (Crockford alphabet, 6 chars) with prefix from config
    - loop until `store.ticket_exists(&id)` is false
  - pick content string:
    - if `--content-file`, read file
    - else if stdin not TTY (`std::io::stdin().is_terminal() == false`), read stdin
    - else use config template string
  - build `NewTicket { meta, content }` with default type/priority and empty arrays
  - call store.create_ticket
  - print id

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-cli --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/tandem-cli/src/main.rs crates/tandem-cli/tests/ticket_cli_tests.rs
git commit -m "feat(cli): implement tndm ticket create"
```

---

## Task 10: Implement `tndm ticket show` CLI

**Files:**
- Modify: `crates/tandem-cli/src/main.rs`
- Test: `crates/tandem-cli/tests/ticket_cli_tests.rs`

**Step 1: Write the failing test**

Add:

```rust
#[test]
fn ticket_show_prints_meta_state_and_content_sections() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    // Create deterministic ticket via explicit id
    Command::cargo_bin("tndm")
        .unwrap()
        .current_dir(dir.path())
        .args(["ticket", "create", "--id", "TNDM-AAAAAA", "Add foo"])
        .assert()
        .success();

    Command::cargo_bin("tndm")
        .unwrap()
        .current_dir(dir.path())
        .args(["ticket", "show", "TNDM-AAAAAA"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## meta.toml\n"))
        .stdout(predicate::str::contains("id = \"TNDM-AAAAAA\""))
        .stdout(predicate::str::contains("## state.toml\n"))
        .stdout(predicate::str::contains("status = \"todo\""))
        .stdout(predicate::str::contains("## content.md\n"));
}
```

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test -p tandem-cli --locked
```

Expected: FAIL.

**Step 3: Implement show output**

- Use storage `load_ticket`.
- Print the three sections in order.
- For meta/state, print canonical TOML using core formatting (`to_canonical_toml`).

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-cli --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/tandem-cli/src/main.rs crates/tandem-cli/tests/ticket_cli_tests.rs
git commit -m "feat(cli): implement tndm ticket show"
```

---

## Task 11: Implement `tndm ticket list` CLI

**Files:**
- Modify: `crates/tandem-cli/src/main.rs`
- Test: `crates/tandem-cli/tests/ticket_cli_tests.rs`

**Step 1: Write the failing test**

Add:

```rust
#[test]
fn ticket_list_prints_id_status_and_title_sorted() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    for (id, title) in [("TNDM-BBBBBB", "B"), ("TNDM-AAAAAA", "A")] {
        Command::cargo_bin("tndm")
            .unwrap()
            .current_dir(dir.path())
            .args(["ticket", "create", "--id", id, title])
            .assert()
            .success();
    }

    Command::cargo_bin("tndm")
        .unwrap()
        .current_dir(dir.path())
        .args(["ticket", "list"])
        .assert()
        .success()
        .stdout("TNDM-AAAAAA\ttodo\tA\nTNDM-BBBBBB\ttodo\tB\n");
}
```

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test -p tandem-cli --locked
```

Expected: FAIL.

**Step 3: Implement list**

- Call `store.list_ticket_ids()`.
- For each id, call `store.load_ticket(&id)`.
- Print `<id>\t<status>\t<title>`.

**Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p tandem-cli --locked
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/tandem-cli/src/main.rs crates/tandem-cli/tests/ticket_cli_tests.rs
git commit -m "feat(cli): implement tndm ticket list"
```

---

## Task 12: Workspace verification

**Files:**
- (none)

**Step 1: Run full repo checks**

Run:

```bash
mise run check
```

Expected: all steps PASS (fmt, compile, arch, clippy, test).

**Step 2: Commit**

- No commit unless verification required changes.

---

## Execution handoff

Plan complete and saved to `docs/plans/2026-03-03-ticket-commands-implementation-plan.md`.

Two execution options:

1. **Subagent-Driven (this session)** — I dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Parallel Session (separate)** — Open a new session in a worktree and run tasks sequentially with checkpoints using `superpowers:executing-plans`

Which approach do you want?
