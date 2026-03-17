# JSON Output Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `--json` flag to all ticket and awareness commands, with consistent schema-versioned JSON output.

**Architecture:** Add `Serialize` impls to domain types in `tandem-core`. Add a shared `OutputArgs` struct in the CLI crate flattened into each subcommand. CLI-level envelope structs handle JSON shape (flattened meta+state, `content_path` reference). Awareness gets a new text renderer for its default output.

**Tech Stack:** Rust, serde, serde_json, clap (Args/flatten)

---

## Chunk 1: Serialize on domain types

### Task 1: Add Serialize to TicketId

**Files:**
- Modify: `crates/tandem-core/src/ticket.rs:1-41`

- [ ] **Step 1: Write the failing test**

Add to the existing `mod tests` block in `crates/tandem-core/src/ticket.rs`:

```rust
#[test]
fn ticket_id_serializes_as_plain_string() {
    let id = TicketId::parse("TNDM-ABC123").unwrap();
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "\"TNDM-ABC123\"");
}
```

Also add `serde_json` to the test imports at the top of `mod tests`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p tandem-core ticket_id_serializes_as_plain_string`
Expected: FAIL — `Serialize` is not implemented for `TicketId`

- [ ] **Step 3: Write minimal implementation**

Add to `crates/tandem-core/src/ticket.rs`, after the existing `impl fmt::Display for TicketId` block (after line 41):

```rust
impl serde::Serialize for TicketId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p tandem-core ticket_id_serializes_as_plain_string`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/tandem-core/src/ticket.rs
git commit -m "feat(core): add Serialize impl for TicketId"
```

### Task 2: Add Serialize to TicketType, TicketPriority, TicketStatus

**Files:**
- Modify: `crates/tandem-core/src/ticket.rs:43-137`

- [ ] **Step 1: Write the failing tests**

Add to `mod tests` in `crates/tandem-core/src/ticket.rs`:

```rust
#[test]
fn ticket_type_serializes_as_str() {
    assert_eq!(serde_json::to_string(&TicketType::Task).unwrap(), "\"task\"");
    assert_eq!(serde_json::to_string(&TicketType::Bug).unwrap(), "\"bug\"");
    assert_eq!(serde_json::to_string(&TicketType::Feature).unwrap(), "\"feature\"");
    assert_eq!(serde_json::to_string(&TicketType::Chore).unwrap(), "\"chore\"");
    assert_eq!(serde_json::to_string(&TicketType::Epic).unwrap(), "\"epic\"");
}

#[test]
fn ticket_priority_serializes_as_str() {
    assert_eq!(serde_json::to_string(&TicketPriority::P0).unwrap(), "\"p0\"");
    assert_eq!(serde_json::to_string(&TicketPriority::P1).unwrap(), "\"p1\"");
    assert_eq!(serde_json::to_string(&TicketPriority::P2).unwrap(), "\"p2\"");
    assert_eq!(serde_json::to_string(&TicketPriority::P3).unwrap(), "\"p3\"");
    assert_eq!(serde_json::to_string(&TicketPriority::P4).unwrap(), "\"p4\"");
}

#[test]
fn ticket_status_serializes_as_str() {
    assert_eq!(serde_json::to_string(&TicketStatus::Todo).unwrap(), "\"todo\"");
    assert_eq!(serde_json::to_string(&TicketStatus::InProgress).unwrap(), "\"in_progress\"");
    assert_eq!(serde_json::to_string(&TicketStatus::Blocked).unwrap(), "\"blocked\"");
    assert_eq!(serde_json::to_string(&TicketStatus::Done).unwrap(), "\"done\"");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p tandem-core ticket_type_serializes_as_str ticket_priority_serializes_as_str ticket_status_serializes_as_str`
Expected: FAIL — `Serialize` not implemented

- [ ] **Step 3: Write minimal implementation**

Add custom `Serialize` impls for each enum, after each enum's existing `impl` block. Each delegates to `as_str()`:

For `TicketType` (after line 74):
```rust
impl serde::Serialize for TicketType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}
```

For `TicketPriority` (after line 107):
```rust
impl serde::Serialize for TicketPriority {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}
```

For `TicketStatus` (after line 137):
```rust
impl serde::Serialize for TicketStatus {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p tandem-core ticket_type_serializes_as_str ticket_priority_serializes_as_str ticket_status_serializes_as_str`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/tandem-core/src/ticket.rs
git commit -m "feat(core): add Serialize impls for TicketType, TicketPriority, TicketStatus"
```

### Task 3: Add Serialize to TicketMeta and TicketState

**Files:**
- Modify: `crates/tandem-core/src/ticket.rs:139-253`

- [ ] **Step 1: Write the failing tests**

Add to `mod tests` in `crates/tandem-core/src/ticket.rs`:

```rust
#[test]
fn ticket_meta_serializes_with_type_renamed() {
    let id = TicketId::parse("TNDM-TEST01").unwrap();
    let meta = TicketMeta::new(id, "Test title").unwrap();
    let json: serde_json::Value = serde_json::to_value(&meta).unwrap();
    assert_eq!(json["id"], "TNDM-TEST01");
    assert_eq!(json["title"], "Test title");
    assert_eq!(json["type"], "task");
    assert_eq!(json["priority"], "p2");
    assert!(json.get("ticket_type").is_none(), "ticket_type should be renamed to type");
    assert_eq!(json["depends_on"], serde_json::json!([]));
    assert_eq!(json["tags"], serde_json::json!([]));
}

#[test]
fn ticket_state_serializes_all_fields() {
    let state = TicketState::new("2026-03-17T12:00:00Z", 3).unwrap();
    let json: serde_json::Value = serde_json::to_value(&state).unwrap();
    assert_eq!(json["status"], "todo");
    assert_eq!(json["updated_at"], "2026-03-17T12:00:00Z");
    assert_eq!(json["revision"], 3);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p tandem-core ticket_meta_serializes_with_type_renamed ticket_state_serializes_all_fields`
Expected: FAIL — `Serialize` not implemented

- [ ] **Step 3: Write minimal implementation**

Change the `TicketMeta` derive (line 139) to:
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TicketMeta {
    pub id: TicketId,
    pub title: String,
    #[serde(rename = "type")]
    pub ticket_type: TicketType,
    pub priority: TicketPriority,
    pub depends_on: Vec<TicketId>,
    pub tags: Vec<String>,
}
```

Change the `TicketState` derive (line 193) to:
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TicketState {
    pub status: TicketStatus,
    pub updated_at: String,
    pub revision: u64,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p tandem-core ticket_meta_serializes_with_type_renamed ticket_state_serializes_all_fields`
Expected: PASS

- [ ] **Step 5: Run full test suite to confirm no regressions**

Run: `cargo test -p tandem-core`
Expected: All tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/tandem-core/src/ticket.rs
git commit -m "feat(core): add Serialize to TicketMeta and TicketState"
```

## Chunk 2: CLI OutputArgs and JSON envelope structs

### Task 4: Add OutputArgs struct and flatten into subcommands

**Files:**
- Modify: `crates/tandem-cli/src/main.rs:52-110`

- [ ] **Step 1: Add OutputArgs struct and flatten into all ticket subcommands and AwarenessArgs**

Add after the `AwarenessArgs` struct (after line 56):
```rust
#[derive(Args, Debug)]
struct OutputArgs {
    /// Output as JSON instead of human-readable text.
    #[arg(long)]
    json: bool,
}
```

Add `#[command(flatten)] output: OutputArgs` to each ticket subcommand variant:
- `Create { ... }` — add field
- `Show { ... }` — add field
- `List` — change to `List { #[command(flatten)] output: OutputArgs }`
- `Update { ... }` — add field

Add `#[command(flatten)] output: OutputArgs` to `AwarenessArgs`.

Update the `main()` match arms to pass the new `output` field through to handlers. Update handler signatures to accept `output: OutputArgs` (or just `json: bool`). For now, the handlers ignore the flag — behavior change comes in later tasks.

- [ ] **Step 2: Run existing tests to confirm no regressions**

Run: `cargo test -p tandem-cli`
Expected: All existing tests PASS (no behavior change yet)

- [ ] **Step 3: Verify --json flag is accepted by commands**

Run: `cargo run -p tandem-cli -- ticket list --help 2>&1 | grep json`
Expected: Shows `--json` in help output

- [ ] **Step 4: Commit**

```bash
git add crates/tandem-cli/src/main.rs
git commit -m "feat(cli): add OutputArgs with --json flag to all subcommands"
```

### Task 5: Add serde dependency and JSON envelope structs for ticket output

**Files:**
- Modify: `crates/tandem-cli/Cargo.toml`
- Modify: `crates/tandem-cli/src/main.rs`

- [ ] **Step 1: Add serde to CLI crate dependencies**

Add `serde.workspace = true` to `[dependencies]` in `crates/tandem-cli/Cargo.toml` (and `serde_json` to `[dev-dependencies]` for the integration tests):

```toml
[dependencies]
anyhow.workspace = true
clap.workspace = true
rand.workspace = true
serde.workspace = true
serde_json.workspace = true
tandem-core.workspace = true
tandem-repo.workspace = true
tandem-storage.workspace = true
time.workspace = true

[dev-dependencies]
regex = "=1.12.3"
serde_json.workspace = true
tempfile.workspace = true
```

- [ ] **Step 2: Add envelope structs**

Add near the top of `main.rs` (after imports), these CLI-level structs for JSON output. `TicketJsonEntry` is the shared ticket shape (without `schema_version`) used in both single-ticket and list responses. `TicketJson` wraps it with `schema_version` for single-ticket output. `TicketListJson` wraps a `Vec` with `schema_version` for list output.

```rust
use serde::Serialize;

#[derive(Serialize)]
struct TicketJsonEntry<'a> {
    #[serde(flatten)]
    meta: &'a tandem_core::ticket::TicketMeta,
    #[serde(flatten)]
    state: &'a tandem_core::ticket::TicketState,
    content_path: String,
}

#[derive(Serialize)]
struct TicketJson<'a> {
    schema_version: u64,
    #[serde(flatten)]
    ticket: TicketJsonEntry<'a>,
}

#[derive(Serialize)]
struct TicketListJson<'a> {
    schema_version: u64,
    tickets: Vec<TicketJsonEntry<'a>>,
}
```

The `content_path` is built from the ticket ID: `.tndm/tickets/{id}/content.md`.

Add a helper function:

```rust
fn ticket_content_path(id: &tandem_core::ticket::TicketId) -> String {
    format!(".tndm/tickets/{}/content.md", id)
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p tandem-cli`
Expected: Compiles without errors

- [ ] **Step 4: Commit**

```bash
git add crates/tandem-cli/Cargo.toml crates/tandem-cli/src/main.rs
git commit -m "feat(cli): add serde dependency and JSON envelope structs"
```

### Task 6: Implement --json for ticket show

**Files:**
- Modify: `crates/tandem-cli/src/main.rs` (handle_ticket_show)
- Modify: `crates/tandem-cli/tests/ticket_cli_tests.rs`

- [ ] **Step 1: Write the failing test**

Add to `crates/tandem-cli/tests/ticket_cli_tests.rs`:

```rust
#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_show_json_outputs_flat_ticket_with_content_path() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-SHOWJ";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "JSON show test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", ticket_id, "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show --json");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["id"], "TNDM-SHOWJ");
    assert_eq!(json["title"], "JSON show test");
    assert_eq!(json["type"], "task");
    assert_eq!(json["priority"], "p2");
    assert_eq!(json["status"], "todo");
    assert_eq!(json["revision"], 1);
    assert_eq!(json["content_path"], ".tndm/tickets/TNDM-SHOWJ/content.md");
    assert!(json.get("content").is_none(), "content should not be in JSON");
    assert!(json["updated_at"].is_string());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p tandem-cli ticket_show_json_outputs_flat_ticket_with_content_path`
Expected: FAIL — `--json` not yet handled in show

- [ ] **Step 3: Implement --json branch in handle_ticket_show**

Update `handle_ticket_show` to accept the json flag and branch:

```rust
fn handle_ticket_show(id: String, json: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let id = TicketId::parse(id)?;
    let ticket = store
        .load_ticket(&id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if json {
        let envelope = TicketJson {
            schema_version: 1,
            ticket: TicketJsonEntry {
                meta: &ticket.meta,
                state: &ticket.state,
                content_path: ticket_content_path(&ticket.meta.id),
            },
        };
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        print!("## meta.toml\n{}\n", ticket.meta.to_canonical_toml());
        print!("## state.toml\n{}\n", ticket.state.to_canonical_toml());
        print!("## content.md\n{}", ticket.content);
    }
    Ok(())
}
```

Update the `main()` match arm for `Show` to pass `output.json`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p tandem-cli ticket_show_json_outputs_flat_ticket_with_content_path`
Expected: PASS

- [ ] **Step 5: Run existing show test to confirm no regression**

Run: `cargo test -p tandem-cli ticket_show_prints_exact_canonical_sections`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/tandem-cli/src/main.rs crates/tandem-cli/tests/ticket_cli_tests.rs
git commit -m "feat(cli): implement --json for ticket show"
```

### Task 7: Implement --json for ticket list

**Files:**
- Modify: `crates/tandem-cli/src/main.rs` (handle_ticket_list)
- Modify: `crates/tandem-cli/tests/ticket_cli_tests.rs`

- [ ] **Step 1: Write the failing test**

Add to `crates/tandem-cli/tests/ticket_cli_tests.rs`:

```rust
#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_json_outputs_schema_versioned_array() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "First", "--id", "TNDM-1"])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket 1")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Second", "--id", "TNDM-2"])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket 2")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket list --json");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(json["schema_version"], 1);
    let tickets = json["tickets"].as_array().expect("tickets should be an array");
    assert_eq!(tickets.len(), 2);
    assert_eq!(tickets[0]["id"], "TNDM-1");
    assert_eq!(tickets[0]["title"], "First");
    assert_eq!(tickets[0]["content_path"], ".tndm/tickets/TNDM-1/content.md");
    assert!(tickets[0].get("schema_version").is_none(), "individual tickets should not have schema_version");
    assert_eq!(tickets[1]["id"], "TNDM-2");
    assert_eq!(tickets[1]["title"], "Second");
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_list_json_empty_produces_empty_array() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "list", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket list --json");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["tickets"], serde_json::json!([]));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p tandem-cli ticket_list_json`
Expected: FAIL

- [ ] **Step 3: Implement --json branch in handle_ticket_list**

```rust
fn handle_ticket_list(json: bool) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let ids = store
        .list_ticket_ids()
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if json {
        let mut tickets = Vec::new();
        for id in ids {
            let ticket = store
                .load_ticket(&id)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            tickets.push(ticket);
        }
        let envelope = TicketListJson {
            schema_version: 1,
            tickets: tickets
                .iter()
                .map(|t| TicketJsonEntry {
                    meta: &t.meta,
                    state: &t.state,
                    content_path: ticket_content_path(&t.meta.id),
                })
                .collect(),
        };
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        for id in ids {
            let ticket = store
                .load_ticket(&id)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            println!(
                "{}\t{}\t{}",
                id,
                ticket.state.status.as_str(),
                ticket.meta.title
            );
        }
    }

    Ok(())
}
```

Update `main()` match arm for `List` to pass `output.json`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p tandem-cli ticket_list_json`
Expected: PASS

- [ ] **Step 5: Run existing list test to confirm no regression**

Run: `cargo test -p tandem-cli ticket_list_prints_sorted_tab_separated_lines`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/tandem-cli/src/main.rs crates/tandem-cli/tests/ticket_cli_tests.rs
git commit -m "feat(cli): implement --json for ticket list"
```

### Task 8: Implement --json for ticket create

**Files:**
- Modify: `crates/tandem-cli/src/main.rs` (handle_ticket_create)
- Modify: `crates/tandem-cli/tests/ticket_cli_tests.rs`

- [ ] **Step 1: Write the failing test**

Add to `crates/tandem-cli/tests/ticket_cli_tests.rs`:

```rust
#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_json_outputs_full_ticket_envelope() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "JSON create test", "--id", "TNDM-CJ01", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create --json");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["id"], "TNDM-CJ01");
    assert_eq!(json["title"], "JSON create test");
    assert_eq!(json["type"], "task");
    assert_eq!(json["priority"], "p2");
    assert_eq!(json["status"], "todo");
    assert_eq!(json["revision"], 1);
    assert_eq!(json["content_path"], ".tndm/tickets/TNDM-CJ01/content.md");
    assert!(json.get("content").is_none());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p tandem-cli ticket_create_json_outputs_full_ticket_envelope`
Expected: FAIL

- [ ] **Step 3: Implement --json branch in handle_ticket_create**

Update `handle_ticket_create` to capture the return value of `create_ticket` and branch:

```rust
fn handle_ticket_create(
    title: String,
    id: Option<String>,
    content_file: Option<PathBuf>,
    json: bool,
) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root.clone());
    let config = load_config(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;

    let ticket_id = match id {
        Some(value) => TicketId::parse(value)?,
        None => generate_ticket_id(&store, &config.id_prefix)?,
    };

    let content = load_ticket_content(content_file, &config)?;
    let meta = TicketMeta::new(ticket_id, title)?;

    let ticket = store
        .create_ticket(NewTicket { meta, content })
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if json {
        let envelope = TicketJson {
            schema_version: 1,
            ticket: TicketJsonEntry {
                meta: &ticket.meta,
                state: &ticket.state,
                content_path: ticket_content_path(&ticket.meta.id),
            },
        };
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        println!("{}", ticket.meta.id);
    }
    Ok(())
}
```

Update `main()` match arm for `Create` to pass `output.json`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p tandem-cli ticket_create_json_outputs_full_ticket_envelope`
Expected: PASS

- [ ] **Step 5: Run existing create test to confirm no regression**

Run: `cargo test -p tandem-cli ticket_create_prints_generated_id_and_writes_ticket_files`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/tandem-cli/src/main.rs crates/tandem-cli/tests/ticket_cli_tests.rs
git commit -m "feat(cli): implement --json for ticket create"
```

### Task 9: Implement --json for ticket update

**Files:**
- Modify: `crates/tandem-cli/src/main.rs` (handle_ticket_update)
- Modify: `crates/tandem-cli/tests/ticket_cli_tests.rs`

- [ ] **Step 1: Write the failing test**

Add to `crates/tandem-cli/tests/ticket_cli_tests.rs`:

```rust
#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_update_json_outputs_updated_ticket_envelope() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let ticket_id = "TNDM-UJ01";
    Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Update JSON test", "--id", ticket_id])
        .current_dir(repo_root.path())
        .output()
        .expect("create ticket")
        .status
        .success()
        .then_some(())
        .expect("create should succeed");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "update", ticket_id, "--status", "in_progress", "--json"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket update --json");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["id"], "TNDM-UJ01");
    assert_eq!(json["status"], "in_progress");
    assert_eq!(json["revision"], 2);
    assert_eq!(json["content_path"], ".tndm/tickets/TNDM-UJ01/content.md");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p tandem-cli ticket_update_json_outputs_updated_ticket_envelope`
Expected: FAIL

- [ ] **Step 3: Implement --json branch in handle_ticket_update**

Update `handle_ticket_update` to accept `json: bool`, capture the return value of `update_ticket`, and branch:

After the existing `store.update_ticket(&ticket)` call, change to:

```rust
let updated = store
    .update_ticket(&ticket)
    .map_err(|error| anyhow::anyhow!("{error}"))?;

if json {
    let envelope = TicketJson {
        schema_version: 1,
        ticket: TicketJsonEntry {
            meta: &updated.meta,
            state: &updated.state,
            content_path: ticket_content_path(&updated.meta.id),
        },
    };
    println!("{}", serde_json::to_string_pretty(&envelope)?);
} else {
    println!("{ticket_id}");
}
```

Update `main()` match arm for `Update` to pass `output.json`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p tandem-cli ticket_update_json_outputs_updated_ticket_envelope`
Expected: PASS

- [ ] **Step 5: Run existing update tests to confirm no regression**

Run: `cargo test -p tandem-cli ticket_update`
Expected: All PASS

- [ ] **Step 6: Commit**

```bash
git add crates/tandem-cli/src/main.rs crates/tandem-cli/tests/ticket_cli_tests.rs
git commit -m "feat(cli): implement --json for ticket update"
```

## Chunk 3: Awareness text output and --json flag

### Task 10: Add text renderer for awareness and wire --json flag

**Files:**
- Modify: `crates/tandem-cli/src/main.rs` (handle_awareness)
- Modify: `crates/tandem-cli/tests/awareness_cli_tests.rs`

- [ ] **Step 1: Write the failing test for awareness text output**

Add to `crates/tandem-cli/tests/awareness_cli_tests.rs`:

```rust
#[test]
fn awareness_text_output_shows_human_readable_format() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    run_git(repo_root.path(), &["init", "-b", "main"]);
    run_git(repo_root.path(), &["config", "user.name", "Test User"]);
    run_git(
        repo_root.path(),
        &["config", "user.email", "test@example.com"],
    );

    write_ticket(
        repo_root.path(),
        "TNDM-1",
        "Against only",
        "todo",
        "p2",
        &[],
    );
    write_ticket(repo_root.path(), "TNDM-3", "Diverged", "todo", "p2", &[]);
    run_git(repo_root.path(), &["add", "."]);
    run_git(repo_root.path(), &["commit", "-m", "base"]);

    fs::remove_dir_all(
        repo_root
            .path()
            .join(".tndm")
            .join("tickets")
            .join("TNDM-1"),
    )
    .expect("remove TNDM-1");
    write_ticket(
        repo_root.path(),
        "TNDM-2",
        "Current only",
        "todo",
        "p2",
        &[],
    );
    write_ticket(
        repo_root.path(),
        "TNDM-3",
        "Diverged",
        "in_progress",
        "p1",
        &["TNDM-1"],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["awareness", "--against", "HEAD"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm awareness");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("Against: HEAD"));
    assert!(stdout.contains("TNDM-1") && stdout.contains("added (against)"));
    assert!(stdout.contains("TNDM-2") && stdout.contains("added (current)"));
    assert!(stdout.contains("TNDM-3") && stdout.contains("diverged"));
    assert!(stdout.contains("status:") && stdout.contains("in_progress -> todo"));
    assert!(stdout.contains("priority:") && stdout.contains("p1 -> p2"));
}

#[test]
fn awareness_text_output_empty_shows_no_changes() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    run_git(repo_root.path(), &["init", "-b", "main"]);
    run_git(repo_root.path(), &["config", "user.name", "Test User"]);
    run_git(
        repo_root.path(),
        &["config", "user.email", "test@example.com"],
    );

    write_ticket(repo_root.path(), "TNDM-1", "One", "todo", "p2", &[]);
    run_git(repo_root.path(), &["add", "."]);
    run_git(repo_root.path(), &["commit", "-m", "base"]);

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["awareness", "--against", "HEAD"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm awareness");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("Against: HEAD"));
    assert!(stdout.contains("No changes."));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p tandem-cli awareness_text_output`
Expected: FAIL — awareness currently always outputs JSON

- [ ] **Step 3: Implement text renderer and --json branch in handle_awareness**

Add a text rendering function and update `handle_awareness`:

```rust
fn format_awareness_text(report: &tandem_core::awareness::AwarenessReport) -> String {
    use tandem_core::awareness::AwarenessChangeKind;

    let mut output = format!("Against: {}\n\n", report.against);

    if report.tickets.is_empty() {
        output.push_str("No changes.\n");
        return output;
    }

    for ticket in &report.tickets {
        let kind = match &ticket.change {
            AwarenessChangeKind::AddedCurrent => "added (current)",
            AwarenessChangeKind::AddedAgainst => "added (against)",
            AwarenessChangeKind::Diverged => "diverged",
        };
        output.push_str(&format!("{}  {}\n", ticket.id, kind));

        if let Some(ref status) = ticket.fields.status {
            output.push_str(&format!(
                "  status:     {} -> {}\n",
                status.current, status.against
            ));
        }
        if let Some(ref priority) = ticket.fields.priority {
            output.push_str(&format!(
                "  priority:   {} -> {}\n",
                priority.current, priority.against
            ));
        }
        if let Some(ref depends_on) = ticket.fields.depends_on {
            output.push_str(&format!(
                "  depends_on: {:?} -> {:?}\n",
                depends_on.current, depends_on.against
            ));
        }
    }

    output
}

fn handle_awareness(args: AwarenessArgs) -> anyhow::Result<()> {
    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;

    let current_snapshot =
        load_ticket_snapshot(&repo_root).map_err(|error| anyhow::anyhow!("{error}"))?;

    let provider = GitAwarenessProvider::new(repo_root);
    let against_snapshot = match provider
        .materialize_ref_snapshot(&args.against)
        .map_err(|error| anyhow::anyhow!("{error}"))?
    {
        None => tandem_core::awareness::TicketSnapshot::default(),
        Some(snapshot) => load_ticket_snapshot(snapshot.path()).map_err(|error| {
            anyhow::anyhow!(
                "failed to load materialized snapshot for ref `{}`: {}",
                args.against,
                snapshot.sanitize_error_text(&error.to_string())
            )
        })?,
    };

    let report = compare_snapshots(&args.against, &current_snapshot, &against_snapshot);

    if args.output.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print!("{}", format_awareness_text(&report));
    }
    Ok(())
}
```

- [ ] **Step 4: Run new text output tests to verify they pass**

Run: `cargo test -p tandem-cli awareness_text_output`
Expected: PASS

- [ ] **Step 5: Update existing awareness JSON tests to use --json flag**

In `crates/tandem-cli/tests/awareness_cli_tests.rs`:

Change `awareness_prints_empty_json_when_snapshots_match` (line 19-20):
```rust
// Change:
.args(["awareness", "--against", "HEAD"])
// To:
.args(["awareness", "--against", "HEAD", "--json"])
```

Change `awareness_reports_added_current_added_against_and_diverged_sorted` (line 89-90):
```rust
// Change:
.args(["awareness", "--against", "HEAD"])
// To:
.args(["awareness", "--against", "HEAD", "--json"])
```

The `awareness_errors_for_invalid_ref` test (line 151-152) does not assert JSON output, so it does not need `--json`.

- [ ] **Step 6: Run all awareness tests to confirm everything passes**

Run: `cargo test -p tandem-cli awareness`
Expected: All PASS

- [ ] **Step 7: Commit**

```bash
git add crates/tandem-cli/src/main.rs crates/tandem-cli/tests/awareness_cli_tests.rs
git commit -m "feat(cli): add text renderer for awareness, wire --json flag"
```

## Chunk 4: Final validation

### Task 11: Full test suite and lint pass

**Files:** None (validation only)

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 2: Run clippy**

Run: `mise run clippy`
Expected: No warnings or errors

- [ ] **Step 3: Run format check**

Run: `mise run fmt`
Expected: No formatting issues

- [ ] **Step 4: Run architecture check**

Run: `mise run arch`
Expected: PASS

- [ ] **Step 5: Run full check suite**

Run: `mise run check`
Expected: All checks PASS

- [ ] **Step 6: Clean up test ticket from brainstorming**

If the test ticket `TNDM-MYQXK2` still exists, remove it:
```bash
rm -rf .tndm/tickets/TNDM-MYQXK2
```

(It was already cleaned up earlier, but verify.)
