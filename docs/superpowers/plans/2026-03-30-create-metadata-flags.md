# Create Metadata Flags Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:subagent-driven-development (recommended) or superpowers-extended-cc:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `--status`, `--priority`, `--type`, `--tags`, and `--depends-on` flags to `tndm ticket create`, matching the flag set on `update`.

**Architecture:** Extend the `Create` clap variant with the same metadata flags as `Update`. The handler constructs `TicketMeta::new()` then overrides fields from provided flags before calling `store.create_ticket()`. For `--status`, mutate the returned ticket's state and call `store.update_ticket()`. No changes to `tandem-core`.

**Tech Stack:** Rust, clap, existing test infrastructure (integration tests via `Command`).

**User Verification:** NO — no user feedback or sign-off required.

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `crates/tandem-cli/src/main.rs` | Modify | Add flags to `Create` variant, update handler |
| `crates/tandem-cli/tests/ticket_cli_tests.rs` | Modify | Integration tests for new flags |
| `plugin/tndm/skills/ticket/SKILL.md` | Modify | Update create command docs |
| `plugin/tndm/skills/ticket/references/command-reference.md` | Modify | Update create flag reference |
| `plugin/tndm/.claude-plugin/plugin.json` | Modify | Bump version |

---

### Task 1: Add metadata flags to CLI `Create` variant and handler

**Goal:** Extend the `Create` subcommand with `--status`, `--priority`, `--type`, `--tags`, `--depends-on` flags and wire them through the handler.

**Files:**
- Modify: `crates/tandem-cli/src/main.rs:96-116` (Create variant)
- Modify: `crates/tandem-cli/src/main.rs:186-193` (match arm destructuring)
- Modify: `crates/tandem-cli/src/main.rs:269-307` (handler function)

**Acceptance Criteria:**
- [ ] `tndm ticket create --help` shows all five new flags with correct short flags and value descriptions
- [ ] `tndm ticket create "Test" --priority p1 --type bug --tags a,b --depends-on TNDM-X --status in_progress` succeeds
- [ ] Omitting any flag uses the existing defaults (p2, task, [], [], todo)
- [ ] `mise run check` passes (compile, clippy, arch, fmt)

**Verify:** `./tndm-dev ticket create --help` shows all new flags, then `mise run check` passes.

**Steps:**

- [ ] **Step 1: Add flags to `Create` variant**

In the `TicketCommand::Create` variant (lines 96-116), add five new flags after `content` and before `output`. Use the same `#[arg]` annotations as the `Update` variant:

```rust
    /// Create a new ticket.
    Create {
        /// Ticket title.
        title: String,

        /// Optional explicit ticket ID.
        #[arg(long)]
        id: Option<String>,

        /// Optional content markdown file path.
        #[arg(long, conflicts_with = "content")]
        content_file: Option<PathBuf>,

        /// Optional inline content body.
        #[arg(long, conflicts_with = "content_file")]
        content: Option<String>,

        /// Initial status [possible values: todo, in_progress, blocked, done].
        #[arg(long, short)]
        status: Option<TicketStatus>,

        /// Initial priority [possible values: p0, p1, p2, p3, p4].
        #[arg(long, short)]
        priority: Option<TicketPriority>,

        /// Initial ticket type [possible values: task, bug, feature, chore, epic].
        #[arg(long = "type", short = 'T')]
        ticket_type: Option<TicketType>,

        /// Comma-separated tags.
        #[arg(long, short = 'g')]
        tags: Option<String>,

        /// Comma-separated ticket IDs for dependencies.
        #[arg(long, short = 'd')]
        depends_on: Option<String>,

        #[command(flatten)]
        output: OutputArgs,
    },
```

- [ ] **Step 2: Update `main()` match arm**

Update the `TicketCommand::Create` destructuring in `main()` (around line 187-193) to extract the new fields:

```rust
            TicketCommand::Create {
                title,
                id,
                content_file,
                content,
                status,
                priority,
                ticket_type,
                tags,
                depends_on,
                output,
            } => handle_ticket_create(
                title,
                id,
                content_file,
                content,
                status,
                priority,
                ticket_type,
                tags,
                depends_on,
                output.json,
            ),
```

- [ ] **Step 3: Update handler signature and body**

Replace `handle_ticket_create` (lines 269-307) with the extended version. Meta flags are applied before `create_ticket`. Status is applied after by mutating the returned ticket:

```rust
#[allow(clippy::too_many_arguments)]
fn handle_ticket_create(
    title: String,
    id: Option<String>,
    content_file: Option<PathBuf>,
    content: Option<String>,
    status: Option<TicketStatus>,
    priority: Option<TicketPriority>,
    ticket_type: Option<TicketType>,
    tags: Option<String>,
    depends_on: Option<String>,
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

    let content = load_ticket_content(content_file, content, &config)?;
    let mut meta = TicketMeta::new(ticket_id, title)?;

    if let Some(value) = priority {
        meta.priority = value;
    }
    if let Some(value) = ticket_type {
        meta.ticket_type = value;
    }
    if let Some(value) = tags {
        let mut parsed: Vec<String> = if value.trim().is_empty() {
            Vec::new()
        } else {
            value.split(',').map(|s| s.trim().to_string()).collect()
        };
        parsed.sort();
        parsed.dedup();
        meta.tags = parsed;
    }
    if let Some(value) = depends_on {
        let mut parsed: Vec<TicketId> = if value.trim().is_empty() {
            Vec::new()
        } else {
            value
                .split(',')
                .map(|s| TicketId::parse(s.trim()))
                .collect::<Result<Vec<_>, _>>()?
        };
        parsed.sort();
        parsed.dedup();
        meta.depends_on = parsed;
    }

    let mut ticket = store
        .create_ticket(NewTicket { meta, content })
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if let Some(value) = status {
        ticket.state.status = value;
        ticket = store
            .update_ticket(&ticket)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
    }

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

Note: `--status` triggers a second write (`update_ticket`) because `create_ticket` always creates `TicketState::initial()` with `status = todo`. This is acceptable — the alternative would be changing `NewTicket` to carry initial state, which violates the spec's "no core changes" constraint. The double-write only happens when `--status` is explicitly passed.

- [ ] **Step 4: Verify compilation**

Run: `mise run check`
Expected: All checks pass (compile, clippy, arch, fmt).

- [ ] **Step 5: Commit**

```bash
git add crates/tandem-cli/src/main.rs
git commit -m "feat(cli): add metadata flags to ticket create"
```

---

### Task 2: Add integration tests for create flags

**Goal:** Verify each new flag works correctly, including combined usage and edge cases.

**Files:**
- Modify: `crates/tandem-cli/tests/ticket_cli_tests.rs`

**Acceptance Criteria:**
- [ ] `tndm ticket create "X" --id TNDM-TEST --priority p0 --type bug --tags a,b,c --depends-on TNDM-X,TNDM-Y --status in_progress` creates a ticket with all fields set correctly
- [ ] `tndm ticket create "X" --id TNDM-TEST --tags ""` creates a ticket with empty tags (matches default behavior)
- [ ] `tndm ticket create "X" --id TNDM-TEST --depends-on "bad-id"` fails with a parse error
- [ ] `tndm ticket create "X" --id TNDM-TEST --priority p9` fails with invalid priority error
- [ ] `mise run test` passes

**Verify:** `cargo test --test ticket_cli_tests` → all tests pass

**Steps:**

- [ ] **Step 1: Write test for all flags combined**

Add at end of `ticket_cli_tests.rs`:

```rust
#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_with_all_metadata_flags() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    // Create prerequisite tickets for depends_on
    for id in ["TNDM-A1", "TNDM-A2"] {
        Command::new(env!("CARGO_BIN_EXE_tndm"))
            .args(["ticket", "create", "prereq", "--id", id])
            .current_dir(repo_root.path())
            .output()
            .expect("create prereq ticket")
            .status
            .success()
            .then_some(())
            .expect("create should succeed");
    }

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "Full flags test",
            "--id",
            "TNDM-FL01",
            "--priority",
            "p0",
            "--type",
            "bug",
            "--tags",
            "auth,security",
            "--depends-on",
            "TNDM-A1,TNDM-A2",
            "--status",
            "in_progress",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create with all flags");

    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", "TNDM-FL01"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("priority = \"p0\""),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("type = \"bug\""),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("tags = [\"auth\", \"security\"]"),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("depends_on = [\"TNDM-A1\", \"TNDM-A2\"]"),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("status = \"in_progress\""),
        "show output was: {show_stdout}"
    );
}
```

- [ ] **Step 2: Write test for individual priority flag**

```rust
#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_with_priority_flag() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Priority test", "--id", "TNDM-PR01", "--priority", "p1"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let show = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "show", "TNDM-PR01"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket show");

    let show_stdout = String::from_utf8(show.stdout).expect("stdout should be UTF-8");
    assert!(
        show_stdout.contains("priority = \"p1\""),
        "show output was: {show_stdout}"
    );
    // Defaults still apply for unset fields
    assert!(
        show_stdout.contains("type = \"task\""),
        "show output was: {show_stdout}"
    );
    assert!(
        show_stdout.contains("status = \"todo\""),
        "show output was: {show_stdout}"
    );
}
```

- [ ] **Step 3: Write test for rejection of invalid values**

```rust
#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_rejects_invalid_priority() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args(["ticket", "create", "Bad priority", "--id", "TNDM-BP01", "--priority", "p9"])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create");

    assert!(!output.status.success(), "invalid priority should fail");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("invalid ticket priority"),
        "stderr was: {stderr}"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn ticket_create_rejects_invalid_depends_on() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let output = Command::new(env!("CARGO_BIN_EXE_tndm"))
        .args([
            "ticket",
            "create",
            "Bad depends",
            "--id",
            "TNDM-BD01",
            "--depends-on",
            "not-a-valid-id",
        ])
        .current_dir(repo_root.path())
        .output()
        .expect("run tndm ticket create");

    assert!(!output.status.success(), "invalid depends_on should fail");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("invalid ticket id"),
        "stderr was: {stderr}"
    );
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test ticket_cli_tests`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/tandem-cli/tests/ticket_cli_tests.rs
git commit -m "test(cli): add tests for create metadata flags"
```

---

### Task 3: Update plugin documentation

**Goal:** Update SKILL.md and command-reference.md to reflect that `create` now accepts metadata flags.

**Files:**
- Modify: `plugin/tndm/skills/ticket/SKILL.md`
- Modify: `plugin/tndm/skills/ticket/references/command-reference.md`
- Modify: `plugin/tndm/.claude-plugin/plugin.json`

**Acceptance Criteria:**
- [ ] SKILL.md shows metadata flags on `create` examples
- [ ] command-reference.md documents all new flags on `create`
- [ ] Workflow protocol no longer requires immediate `update` for status when set at creation
- [ ] Plugin version bumped

**Verify:** Read both files and confirm all new flags are documented.

**Steps:**

- [ ] **Step 1: Update SKILL.md Create section**

Replace the `### Create` section (lines 72-91) in `plugin/tndm/skills/ticket/SKILL.md`:

```markdown
### Create

```sh
# Minimal — auto-generates ID, defaults to todo/p2/task
tndm ticket create "<title>"

# With metadata flags (set priority, type, tags, deps at creation)
tndm ticket create "Fix login timeout" \
  --priority p1 --type bug --tags auth,security \
  --depends-on TNDM-AAAAAA,TNDM-BBBBBB

# Start as in_progress in one command (no separate update needed)
tndm ticket create "Urgent hotfix" \
  --status in_progress --priority p0 --type bug
```

With optional content body (use a heredoc — do **not** create temporary files):

```sh
tndm ticket create "Implement OAuth flow" <<'EOF'
## Description

Add OAuth 2.0 authorization code flow.

## Acceptance

- Users can sign in with Google
EOF
```
```

- [ ] **Step 2: Update SKILL.md Workflow Protocol section**

Update the "Create a ticket before starting work" section (lines 24-33) to show that status can be set at creation:

Replace:
```markdown
### 1. Create a ticket before starting work

```sh
tndm ticket create "Brief title describing the task"
```

Note the returned ticket ID (format: `TNDM-XXXXXX`). Immediately update status:

```sh
tndm ticket update <ID> --status in_progress
```
```

With:
```markdown
### 1. Create a ticket before starting work

```sh
tndm ticket create "Brief title describing the task" --status in_progress
```

Note the returned ticket ID (format: `TNDM-XXXXXX`).

Or create as `todo` (the default) and update status later:

```sh
tndm ticket create "Brief title describing the task"
tndm ticket update <ID> --status in_progress
```
```

- [ ] **Step 3: Update command-reference.md create section**

Replace the `## tndm ticket create` section (lines 5-55) in `plugin/tndm/skills/ticket/references/command-reference.md`:

```markdown
## tndm ticket create

Create a new ticket. Prints the ticket ID (text) or full ticket (JSON).

```sh
tndm ticket create <TITLE> [OPTIONS]

Options:
      --id <ID>                  Explicit ticket ID (e.g. TNDM-A1B2C3). Auto-generated if omitted.
  -s, --status <STATUS>          Initial status. Values: todo | in_progress | blocked | done
  -p, --priority <PRIORITY>      Initial priority. Values: p0 | p1 | p2 | p3 | p4
  -T, --type <TYPE>              Initial type. Values: task | bug | feature | chore | epic
  -g, --tags <TAGS>              Comma-separated tags.
  -d, --depends-on <IDS>         Comma-separated ticket IDs for dependencies.
      --content <BODY>           Inline content body.
      --content-file <PATH>      Load ticket body from a markdown file.
      --json                     Output the created ticket as JSON.

Content can also be piped via stdin (heredoc recommended for agents).
--content, --content-file, and stdin are mutually exclusive.
```

Defaults when flags are omitted: status=todo, priority=p2, type=task, tags=[], depends_on=[].

Examples:

```sh
# Minimal — auto-generates ID
tndm ticket create "Refactor auth module"

# With metadata — set everything at creation
tndm ticket create "Fix login timeout" \
  --priority p1 --type bug --tags auth,security \
  --depends-on TNDM-B2C3D4 --status in_progress

# With explicit ID
tndm ticket create "Fix login redirect" --id TNDM-FIX001

# With content via heredoc (preferred for agents — no temp files needed)
tndm ticket create "Implement OAuth flow" --type feature <<'EOF'
## Description

Add OAuth 2.0 authorization code flow.
EOF

# With content from file (when content already exists on disk)
tndm ticket create "Implement OAuth flow" --content-file /tmp/ticket-body.md
```
```

- [ ] **Step 4: Bump plugin version**

In `plugin/tndm/.claude-plugin/plugin.json`, bump the `version` field by one patch version.

- [ ] **Step 5: Commit**

```bash
git add plugin/tndm/skills/ticket/SKILL.md plugin/tndm/skills/ticket/references/command-reference.md plugin/tndm/.claude-plugin/plugin.json
git commit -m "docs(plugin): update create command docs with metadata flags"
```
