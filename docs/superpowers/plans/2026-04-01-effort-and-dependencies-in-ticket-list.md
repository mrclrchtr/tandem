# Effort and Dependencies in Ticket List — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:subagent-driven-development (recommended) or superpowers-extended-cc:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `TicketEffort` T-shirt-size enum to the data model and surface both `effort` and `depends_on` in `tndm ticket list` plain text output, `--json`, and `tndm awareness`.

**Architecture:** `TicketEffort` is defined in `tandem-core` and stored as an optional field in `TicketMeta`. `tandem-storage` deserializes it from TOML. `tandem-cli` exposes `-e, --effort` flags and widens the list output from 4 to 6 tab-separated columns.

**Tech Stack:** Rust, Clap 4, serde, serde_json, toml

**User Verification:** NO

---

## File Map

| File | Change |
|------|--------|
| `crates/tandem-core/src/ticket.rs` | Add `TicketEffort` enum; add `effort: Option<TicketEffort>` to `TicketMeta`; update `new()` and `to_canonical_toml()`; add tests |
| `crates/tandem-core/src/awareness.rs` | Add `effort` field to `AwarenessFieldDiffs`; update `between()` and `is_empty()` |
| `crates/tandem-storage/src/lib.rs` | Add `effort: Option<String>` to `RawTicketMeta`; import and parse `TicketEffort` in `load_ticket()` |
| `crates/tandem-cli/src/main.rs` | Import `TicketEffort`; add `-e, --effort` to `Create` and `Update` variants; update `handle_ticket_create`, `handle_ticket_update`, `handle_ticket_list`, `format_awareness_text` |
| `plugins/tndm/skills/ticket/references/command-reference.md` | Document `-e, --effort` flag and add Effort reference table |

---

## Task 1: Add TicketEffort to tandem-core

**Goal:** Define the `TicketEffort` type, attach it to `TicketMeta`, and track it in awareness diffs.

**Files:**
- Modify: `crates/tandem-core/src/ticket.rs`
- Modify: `crates/tandem-core/src/awareness.rs`

**Acceptance Criteria:**
- [ ] `TicketEffort` parses `xs|s|m|l|xl` case-insensitively and rejects unknown values
- [ ] `Display` renders lowercase; `Serialize` produces a lowercase JSON string
- [ ] `TicketMeta::to_canonical_toml()` omits the `effort` key when `None`, emits `effort = "m"` when `Some(M)`
- [ ] `TicketMeta` JSON serialization includes `"effort": null` when unset, `"effort": "m"` when set
- [ ] `AwarenessFieldDiffs` detects and serializes effort changes
- [ ] `cargo test -p tandem-core` passes

**Verify:** `cargo test -p tandem-core 2>&1 | tail -5` → `test result: ok. N passed`

**Steps:**

- [ ] **Step 1: Write failing tests for TicketEffort**

Add these tests inside the existing `mod tests { ... }` block at the bottom of `crates/tandem-core/src/ticket.rs` (after line 690, before the closing `}`):

```rust
    #[test]
    fn ticket_effort_parse_and_as_str_roundtrip() {
        for value in ["xs", "s", "m", "l", "xl"] {
            assert_eq!(
                TicketEffort::parse(value)
                    .expect("effort should parse")
                    .as_str(),
                value
            );
        }
    }

    #[test]
    fn ticket_effort_parse_is_case_insensitive() {
        assert_eq!(TicketEffort::parse("XS").unwrap(), TicketEffort::Xs);
        assert_eq!(TicketEffort::parse("S").unwrap(), TicketEffort::S);
        assert_eq!(TicketEffort::parse("M").unwrap(), TicketEffort::M);
        assert_eq!(TicketEffort::parse("L").unwrap(), TicketEffort::L);
        assert_eq!(TicketEffort::parse("XL").unwrap(), TicketEffort::Xl);
    }

    #[test]
    fn ticket_effort_parse_rejects_unknown_value() {
        let error = TicketEffort::parse("huge").expect_err("effort should be rejected");
        assert_eq!(
            error.message(),
            "invalid ticket effort [possible values: xs, s, m, l, xl]"
        );
    }

    #[test]
    fn ticket_effort_display_renders_lowercase() {
        assert_eq!(format!("{}", TicketEffort::Xs), "xs");
        assert_eq!(format!("{}", TicketEffort::S), "s");
        assert_eq!(format!("{}", TicketEffort::M), "m");
        assert_eq!(format!("{}", TicketEffort::L), "l");
        assert_eq!(format!("{}", TicketEffort::Xl), "xl");
    }

    #[test]
    fn ticket_effort_serializes_as_str() {
        assert_eq!(serde_json::to_string(&TicketEffort::Xs).unwrap(), "\"xs\"");
        assert_eq!(serde_json::to_string(&TicketEffort::S).unwrap(), "\"s\"");
        assert_eq!(serde_json::to_string(&TicketEffort::M).unwrap(), "\"m\"");
        assert_eq!(serde_json::to_string(&TicketEffort::L).unwrap(), "\"l\"");
        assert_eq!(serde_json::to_string(&TicketEffort::Xl).unwrap(), "\"xl\"");
    }

    #[test]
    fn meta_without_effort_canonical_toml_unchanged() {
        let id = TicketId::parse("TNDM-4K7D9Q").expect("id should parse");
        let meta = TicketMeta::new(id, "Add foo").expect("meta should be valid");

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

    #[test]
    fn meta_with_effort_formats_as_canonical_toml() {
        let id = TicketId::parse("TNDM-4K7D9Q").expect("id should parse");
        let mut meta = TicketMeta::new(id, "Add foo").expect("meta should be valid");
        meta.effort = Some(TicketEffort::M);

        assert_eq!(
            meta.to_canonical_toml(),
            concat!(
                "schema_version = 1\n",
                "id = \"TNDM-4K7D9Q\"\n",
                "title = \"Add foo\"\n",
                "\n",
                "type = \"task\"\n",
                "priority = \"p2\"\n",
                "effort = \"m\"\n",
                "\n",
                "depends_on = []\n",
                "tags = []\n",
            )
        );
    }

    #[test]
    fn ticket_meta_serializes_effort_as_null_when_absent() {
        let id = TicketId::parse("TNDM-TEST01").unwrap();
        let meta = TicketMeta::new(id, "Test title").unwrap();
        let json: serde_json::Value = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["effort"], serde_json::Value::Null);
    }

    #[test]
    fn ticket_meta_serializes_effort_when_present() {
        let id = TicketId::parse("TNDM-TEST01").unwrap();
        let mut meta = TicketMeta::new(id, "Test title").unwrap();
        meta.effort = Some(TicketEffort::M);
        let json: serde_json::Value = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["effort"], "m");
    }
```

Run: `cargo test -p tandem-core 2>&1 | tail -10`
Expected: compile errors — `TicketEffort` not defined yet, `meta.effort` not found.

- [ ] **Step 2: Define the TicketEffort type**

In `crates/tandem-core/src/ticket.rs`, add the following block after the `TicketStatus` impl block (after line 207, before `#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]` on `TicketMeta`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketEffort {
    Xs,
    S,
    M,
    L,
    Xl,
}

impl TicketEffort {
    pub fn parse(value: &str) -> Result<Self, ValidationError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "xs" => Ok(Self::Xs),
            "s" => Ok(Self::S),
            "m" => Ok(Self::M),
            "l" => Ok(Self::L),
            "xl" => Ok(Self::Xl),
            _ => Err(ValidationError::new(
                "invalid ticket effort [possible values: xs, s, m, l, xl]",
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::S => "s",
            Self::M => "m",
            Self::L => "l",
            Self::Xl => "xl",
        }
    }
}

impl FromStr for TicketEffort {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for TicketEffort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl serde::Serialize for TicketEffort {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}
```

- [ ] **Step 3: Add effort field to TicketMeta**

In `crates/tandem-core/src/ticket.rs`, replace the `TicketMeta` struct definition (lines 209–218):

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TicketMeta {
    pub id: TicketId,
    pub title: String,
    #[serde(rename = "type")]
    pub ticket_type: TicketType,
    pub priority: TicketPriority,
    pub effort: Option<TicketEffort>,
    pub depends_on: Vec<TicketId>,
    pub tags: Vec<String>,
}
```

In `TicketMeta::new()`, add `effort: None` to the struct literal (after `priority`):

```rust
Ok(Self {
    id,
    title,
    ticket_type: TicketType::default(),
    priority: TicketPriority::default(),
    effort: None,
    depends_on: Vec::new(),
    tags: Vec::new(),
})
```

- [ ] **Step 4: Update to_canonical_toml() to emit effort**

In `crates/tandem-core/src/ticket.rs`, replace the `to_canonical_toml` method body. Find the lines (around 237–261):

```rust
    pub fn to_canonical_toml(&self) -> String {
        let mut output = String::new();
        output.push_str("schema_version = 1\n");
        output.push_str("id = ");
        output.push_str(&toml_basic_string(self.id.as_str()));
        output.push('\n');
        output.push_str("title = ");
        output.push_str(&toml_basic_string(&self.title));
        output.push_str("\n\n");
        output.push_str("type = ");
        output.push_str(&toml_basic_string(self.ticket_type.as_str()));
        output.push('\n');
        output.push_str("priority = ");
        output.push_str(&toml_basic_string(self.priority.as_str()));
        output.push_str("\n\n");
        output.push_str("depends_on = ");
        output.push_str(&toml_string_array(
            self.depends_on.iter().map(TicketId::as_str),
        ));
        output.push('\n');
        output.push_str("tags = ");
        output.push_str(&toml_string_array(self.tags.iter().map(String::as_str)));
        output.push('\n');
        output
    }
```

Replace with:

```rust
    pub fn to_canonical_toml(&self) -> String {
        let mut output = String::new();
        output.push_str("schema_version = 1\n");
        output.push_str("id = ");
        output.push_str(&toml_basic_string(self.id.as_str()));
        output.push('\n');
        output.push_str("title = ");
        output.push_str(&toml_basic_string(&self.title));
        output.push_str("\n\n");
        output.push_str("type = ");
        output.push_str(&toml_basic_string(self.ticket_type.as_str()));
        output.push('\n');
        output.push_str("priority = ");
        output.push_str(&toml_basic_string(self.priority.as_str()));
        output.push('\n');
        if let Some(effort) = self.effort {
            output.push_str("effort = ");
            output.push_str(&toml_basic_string(effort.as_str()));
            output.push('\n');
        }
        output.push('\n');
        output.push_str("depends_on = ");
        output.push_str(&toml_string_array(
            self.depends_on.iter().map(TicketId::as_str),
        ));
        output.push('\n');
        output.push_str("tags = ");
        output.push_str(&toml_string_array(self.tags.iter().map(String::as_str)));
        output.push('\n');
        output
    }
```

- [ ] **Step 5: Update meta_formats_as_canonical_toml test to match new format**

The existing `meta_formats_as_canonical_toml` test at line 515 checks the exact TOML output for a ticket with no effort. After Step 4, the blank line placement changed slightly — `priority` is followed directly by the optional `effort` line, then a blank line. For `None` effort the blank line is still emitted via `output.push('\n')`. The expected string in the existing test must remain the same:

```
"type = \"task\"\n",
"priority = \"p2\"\n",
"\n",
"depends_on = []\n",
```

Verify that the test still matches the new output by running:

Run: `cargo test -p tandem-core meta_formats_as_canonical_toml 2>&1`
Expected: `test ticket::tests::meta_formats_as_canonical_toml ... ok`

If it fails, update the expected string in `meta_formats_as_canonical_toml` to match the actual output shown in the failure message.

- [ ] **Step 6: Add effort tracking to AwarenessFieldDiffs**

In `crates/tandem-core/src/awareness.rs`, replace the `AwarenessFieldDiffs` struct (lines 72–86):

```rust
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct AwarenessFieldDiffs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AwarenessFieldDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<AwarenessFieldDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<AwarenessFieldDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<AwarenessFieldDiff>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ticket_type: Option<AwarenessFieldDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<AwarenessVecDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<AwarenessVecDiff>,
}
```

Replace the `between()` method body (lines 89–125):

```rust
    fn between(current: &Ticket, against: &Ticket) -> Option<Self> {
        let status = diff_value(current.state.status.as_str(), against.state.status.as_str());
        let priority = diff_value(
            current.meta.priority.as_str(),
            against.meta.priority.as_str(),
        );
        let effort = diff_value(
            current.meta.effort.as_ref().map(TicketEffort::as_str).unwrap_or("-"),
            against.meta.effort.as_ref().map(TicketEffort::as_str).unwrap_or("-"),
        );
        let title = diff_value(&current.meta.title, &against.meta.title);
        let ticket_type = diff_value(
            current.meta.ticket_type.as_str(),
            against.meta.ticket_type.as_str(),
        );

        let current_depends_on = canonicalize_depends_on(&current.meta.depends_on);
        let against_depends_on = canonicalize_depends_on(&against.meta.depends_on);
        let depends_on = (current_depends_on != against_depends_on).then_some(AwarenessVecDiff {
            current: current_depends_on,
            against: against_depends_on,
        });

        let current_tags = canonicalize_tags(&current.meta.tags);
        let against_tags = canonicalize_tags(&against.meta.tags);
        let tags = (current_tags != against_tags).then_some(AwarenessVecDiff {
            current: current_tags,
            against: against_tags,
        });

        let diffs = Self {
            status,
            priority,
            effort,
            title,
            ticket_type,
            depends_on,
            tags,
        };

        (!diffs.is_empty()).then_some(diffs)
    }
```

Add `use crate::ticket::TicketEffort;` to the imports at the top of `awareness.rs`:

```rust
use crate::ticket::{Ticket, TicketEffort, TicketId};
```

Replace `is_empty()` (lines 127–135):

```rust
    fn is_empty(&self) -> bool {
        self.status.is_none()
            && self.priority.is_none()
            && self.effort.is_none()
            && self.title.is_none()
            && self.ticket_type.is_none()
            && self.depends_on.is_none()
            && self.tags.is_none()
    }
```

- [ ] **Step 7: Run tests and commit**

Run: `cargo test -p tandem-core 2>&1 | tail -5`
Expected: `test result: ok. N passed; 0 failed`

```bash
git add crates/tandem-core/src/ticket.rs crates/tandem-core/src/awareness.rs
git commit -m "feat(core): add TicketEffort type and effort field to TicketMeta"
```

```json:metadata
{"files": ["crates/tandem-core/src/ticket.rs", "crates/tandem-core/src/awareness.rs"], "verifyCommand": "cargo test -p tandem-core 2>&1 | tail -5", "acceptanceCriteria": ["TicketEffort parses xs|s|m|l|xl", "effort absent from TOML when None", "effort present in TOML when Some", "awareness tracks effort diffs"], "requiresUserVerification": false}
```

---

## Task 2: Update tandem-storage to deserialize effort

**Goal:** Load the `effort` field from `meta.toml` into `TicketMeta.effort`.

**Files:**
- Modify: `crates/tandem-storage/src/lib.rs`

**Acceptance Criteria:**
- [ ] A `meta.toml` with `effort = "m"` loads with `meta.effort == Some(TicketEffort::M)`
- [ ] A `meta.toml` without an `effort` key loads with `meta.effort == None`
- [ ] An invalid value like `effort = "huge"` returns a `StorageError`
- [ ] `cargo test -p tandem-storage` passes

**Verify:** `cargo test -p tandem-storage 2>&1 | tail -5` → `test result: ok. N passed`

**Steps:**

- [ ] **Step 1: Add effort to RawTicketMeta**

In `crates/tandem-storage/src/lib.rs`, replace the `RawTicketMeta` struct (lines 84–94):

```rust
#[derive(Debug, Deserialize)]
struct RawTicketMeta {
    schema_version: Option<u32>,
    id: String,
    title: String,
    #[serde(rename = "type")]
    ticket_type: Option<String>,
    priority: Option<String>,
    effort: Option<String>,
    depends_on: Option<Vec<String>>,
    tags: Option<Vec<String>>,
}
```

- [ ] **Step 2: Import TicketEffort**

In `crates/tandem-storage/src/lib.rs`, replace the `tandem_core` import (lines 13–17):

```rust
use tandem_core::{
    awareness::TicketSnapshot,
    ports::TicketStore,
    ticket::{
        NewTicket, Ticket, TicketEffort, TicketId, TicketMeta, TicketPriority, TicketState,
        TicketStatus, TicketType,
    },
};
```

- [ ] **Step 3: Parse effort in load_ticket()**

In `crates/tandem-storage/src/lib.rs`, inside `load_ticket()`, add effort parsing after the `priority` block (after line 387, before `meta.depends_on = depends_on;`):

```rust
        if let Some(effort) = raw_meta.effort {
            meta.effort = Some(TicketEffort::parse(&effort).map_err(|error| {
                StorageError::new(format!(
                    "invalid effort in {}: {error}",
                    meta_path.display()
                ))
            })?);
        }
```

- [ ] **Step 4: Run tests and commit**

Run: `cargo test -p tandem-storage 2>&1 | tail -5`
Expected: `test result: ok. N passed; 0 failed`

Also run the full test suite to make sure nothing is broken:

Run: `cargo test 2>&1 | tail -5`
Expected: `test result: ok. N passed; 0 failed`

```bash
git add crates/tandem-storage/src/lib.rs
git commit -m "feat(storage): deserialize effort field from meta.toml"
```

```json:metadata
{"files": ["crates/tandem-storage/src/lib.rs"], "verifyCommand": "cargo test 2>&1 | tail -5", "acceptanceCriteria": ["effort=Some loaded from TOML with effort key", "effort=None when key absent", "invalid effort value returns StorageError"], "requiresUserVerification": false}
```

---

## Task 3: Update tandem-cli — flags, list output, awareness text

**Goal:** Expose `--effort` on `create`/`update`, widen `ticket list` to 6 columns, and show effort diffs in `awareness`.

**Files:**
- Modify: `crates/tandem-cli/src/main.rs`

**Acceptance Criteria:**
- [ ] `tndm ticket create "Title" --effort m` stores `effort = "m"` in `meta.toml`
- [ ] `tndm ticket update TICKET-ID --effort xl` updates effort
- [ ] `tndm ticket update TICKET-ID --effort ""` is not valid (clap rejects unknown enum values)
- [ ] `tndm ticket list` plain text has 6 tab-separated columns: `ID STATUS PRIORITY EFFORT DEPS TITLE`
- [ ] Effort column shows `-` when unset
- [ ] `tndm ticket list --json` includes `"effort"` key
- [ ] `tndm awareness --against main` text output shows `effort` diffs
- [ ] `cargo test -p tandem-cli` passes

**Verify:** `cargo test -p tandem-cli 2>&1 | tail -5` → `test result: ok. N passed`

**Steps:**

- [ ] **Step 1: Import TicketEffort in CLI**

In `crates/tandem-cli/src/main.rs`, replace the `tandem_core` import (line 11–15):

```rust
use tandem_core::{
    awareness::compare_snapshots,
    ports::TicketStore,
    ticket::{NewTicket, TicketEffort, TicketId, TicketMeta, TicketPriority, TicketStatus, TicketType},
};
```

- [ ] **Step 2: Add --effort flag to TicketCommand::Create**

In `crates/tandem-cli/src/main.rs`, inside the `TicketCommand::Create { ... }` variant (after the `depends_on` field, before `output`), add:

```rust
        /// Effort estimate [possible values: xs, s, m, l, xl].
        #[arg(long, short = 'e')]
        effort: Option<TicketEffort>,
```

The full Create variant becomes:

```rust
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

        /// Effort estimate [possible values: xs, s, m, l, xl].
        #[arg(long, short = 'e')]
        effort: Option<TicketEffort>,

        #[command(flatten)]
        output: OutputArgs,
    },
```

- [ ] **Step 3: Add --effort flag to TicketCommand::Update**

In `crates/tandem-cli/src/main.rs`, inside `TicketCommand::Update { ... }` (after `depends_on`, before `content_file`), add:

```rust
        /// Effort estimate [possible values: xs, s, m, l, xl].
        #[arg(long, short = 'e')]
        effort: Option<TicketEffort>,
```

The full Update variant becomes:

```rust
    /// Update an existing ticket.
    #[command(arg_required_else_help = true)]
    Update {
        /// Ticket ID to update.
        id: String,

        /// New status [possible values: todo, in_progress, blocked, done].
        #[arg(long, short)]
        status: Option<TicketStatus>,

        /// New priority [possible values: p0, p1, p2, p3, p4].
        #[arg(long, short)]
        priority: Option<TicketPriority>,

        /// New title.
        #[arg(long, short)]
        title: Option<String>,

        /// New ticket type [possible values: task, bug, feature, chore, epic].
        #[arg(long = "type", short = 'T')]
        ticket_type: Option<TicketType>,

        /// Comma-separated tags (replaces existing list, empty string clears).
        #[arg(long, short = 'g')]
        tags: Option<String>,

        /// Comma-separated ticket IDs for dependencies (replaces existing list, empty string clears).
        #[arg(long, short = 'd')]
        depends_on: Option<String>,

        /// Effort estimate [possible values: xs, s, m, l, xl].
        #[arg(long, short = 'e')]
        effort: Option<TicketEffort>,

        /// Markdown file replacing content.
        #[arg(long, conflicts_with = "update_content")]
        content_file: Option<PathBuf>,

        /// Inline content body replacing existing content.
        #[arg(
            long = "content",
            id = "update_content",
            conflicts_with = "content_file"
        )]
        content: Option<String>,

        #[command(flatten)]
        output: OutputArgs,
    },
```

- [ ] **Step 4: Thread effort through the main() match arm for Create**

In `crates/tandem-cli/src/main.rs`, in the `main()` function, the `TicketCommand::Create` match arm (around lines 207–229) currently destructs without `effort`. Update it to include `effort` and pass it to the handler:

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
                effort,
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
                effort,
                output.json,
            ),
```

- [ ] **Step 5: Thread effort through the main() match arm for Update**

In `crates/tandem-cli/src/main.rs`, the `TicketCommand::Update` match arm (around lines 232–254) currently destructs without `effort`. Update it:

```rust
            TicketCommand::Update {
                id,
                status,
                priority,
                title,
                ticket_type,
                tags,
                depends_on,
                effort,
                content_file,
                content,
                output,
            } => handle_ticket_update(
                id,
                status,
                priority,
                title,
                ticket_type,
                tags,
                depends_on,
                effort,
                content_file,
                content,
                output.json,
            ),
```

- [ ] **Step 6: Update handle_ticket_create signature and body**

Replace the `handle_ticket_create` function signature and add the effort assignment (after the `depends_on` block, before `store.create_ticket`). The full updated function:

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
    effort: Option<TicketEffort>,
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
    if let Some(value) = effort {
        meta.effort = Some(value);
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

- [ ] **Step 7: Update handle_ticket_update signature, no_explicit_update checks, and body**

Replace the `handle_ticket_update` function. The key changes are: add `effort: Option<TicketEffort>` parameter; add `&& effort.is_none()` to both `no_explicit_update` and the bail check; add the effort assignment after the `depends_on` block.

```rust
#[allow(clippy::too_many_arguments)]
fn handle_ticket_update(
    id: String,
    status: Option<TicketStatus>,
    priority: Option<TicketPriority>,
    title: Option<String>,
    ticket_type: Option<TicketType>,
    tags: Option<String>,
    depends_on: Option<String>,
    effort: Option<TicketEffort>,
    content_file: Option<PathBuf>,
    content: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let no_explicit_update = content_file.is_none()
        && content.is_none()
        && status.is_none()
        && priority.is_none()
        && title.is_none()
        && ticket_type.is_none()
        && tags.is_none()
        && depends_on.is_none()
        && effort.is_none();
    let stdin_content = if no_explicit_update && !io::stdin().is_terminal() {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        if buf.is_empty() { None } else { Some(buf) }
    } else {
        None
    };

    if status.is_none()
        && priority.is_none()
        && title.is_none()
        && ticket_type.is_none()
        && tags.is_none()
        && depends_on.is_none()
        && effort.is_none()
        && content_file.is_none()
        && content.is_none()
        && stdin_content.is_none()
    {
        anyhow::bail!(
            "at least one update flag is required\n\n  \
             Example: tndm ticket update {id} --status done\n\n  \
             Run 'tndm ticket update --help' for all options"
        );
    }

    let current_dir = env::current_dir().map_err(|error| anyhow::anyhow!("{error}"))?;
    let repo_root = discover_repo_root(&current_dir).map_err(|error| anyhow::anyhow!("{error}"))?;
    let store = FileTicketStore::new(repo_root);
    let ticket_id = TicketId::parse(id)?;
    let mut ticket = store
        .load_ticket(&ticket_id)
        .map_err(|error| anyhow::anyhow!("{error}"))?;

    if let Some(value) = status {
        ticket.state.status = value;
    }
    if let Some(value) = priority {
        ticket.meta.priority = value;
    }
    if let Some(value) = title {
        if value.trim().is_empty() {
            anyhow::bail!("title must not be empty");
        }
        ticket.meta.title = value;
    }
    if let Some(value) = ticket_type {
        ticket.meta.ticket_type = value;
    }
    if let Some(value) = tags {
        let mut parsed: Vec<String> = if value.trim().is_empty() {
            Vec::new()
        } else {
            value.split(',').map(|s| s.trim().to_string()).collect()
        };
        parsed.sort();
        parsed.dedup();
        ticket.meta.tags = parsed;
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
        ticket.meta.depends_on = parsed;
    }
    if let Some(value) = effort {
        ticket.meta.effort = Some(value);
    }
    if let Some(path) = content_file {
        ticket.content = fs::read_to_string(&path)
            .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path.display()))?;
    } else if let Some(value) = content {
        ticket.content = value;
    } else if let Some(value) = stdin_content {
        ticket.content = value;
    }

    ticket.state.revision += 1;
    ticket.state.updated_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| anyhow::anyhow!("failed to format timestamp: {error}"))?;

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
    Ok(())
}
```

- [ ] **Step 8: Update handle_ticket_list to 6 columns**

In `crates/tandem-cli/src/main.rs`, replace the plain-text loop inside `handle_ticket_list` (lines 454–462):

```rust
    } else {
        for ticket in &tickets {
            println!(
                "{}\t{}\t{}\t{}",
                ticket.meta.id,
                ticket.state.status.as_str(),
                ticket.meta.priority.as_str(),
                ticket.meta.title
            );
        }
    }
```

With:

```rust
    } else {
        for ticket in &tickets {
            let deps = ticket
                .meta
                .depends_on
                .iter()
                .map(|id| id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                ticket.meta.id,
                ticket.state.status.as_str(),
                ticket.meta.priority.as_str(),
                ticket.meta.effort.as_ref().map(TicketEffort::as_str).unwrap_or("-"),
                deps,
                ticket.meta.title
            );
        }
    }
```

- [ ] **Step 9: Update format_awareness_text to show effort diffs**

In `crates/tandem-cli/src/main.rs`, inside `format_awareness_text`, add an effort block after the `priority` block (after lines 627–631):

```rust
        if let Some(ref effort) = ticket.fields.effort {
            output.push_str(&format!(
                "  effort:     {} -> {}\n",
                effort.current, effort.against
            ));
        }
```

The updated section of `format_awareness_text` should read:

```rust
        if let Some(ref priority) = ticket.fields.priority {
            output.push_str(&format!(
                "  priority:   {} -> {}\n",
                priority.current, priority.against
            ));
        }
        if let Some(ref effort) = ticket.fields.effort {
            output.push_str(&format!(
                "  effort:     {} -> {}\n",
                effort.current, effort.against
            ));
        }
        if let Some(ref title) = ticket.fields.title {
```

- [ ] **Step 10: Build, run tests, commit**

Run: `cargo build 2>&1 | tail -10`
Expected: no errors

Run: `cargo test 2>&1 | tail -5`
Expected: `test result: ok. N passed; 0 failed`

Run a quick smoke test (if a repo with tickets exists):

```bash
./tndm-dev ticket list
```

Expected: 6 tab-separated columns per row.

```bash
git add crates/tandem-cli/src/main.rs
git commit -m "feat(cli): add --effort flag and widen ticket list to 6 columns"
```

```json:metadata
{"files": ["crates/tandem-cli/src/main.rs"], "verifyCommand": "cargo test 2>&1 | tail -5", "acceptanceCriteria": ["--effort flag accepted by create and update", "ticket list has 6 columns", "effort shows - when unset", "depends_on shown comma-separated", "awareness text shows effort diffs"], "requiresUserVerification": false}
```

---

## Task 4: Update plugin command reference

**Goal:** Document `--effort` in the command reference so agents know the flag exists.

**Files:**
- Modify: `plugins/tndm/skills/ticket/references/command-reference.md`

**Acceptance Criteria:**
- [ ] `-e, --effort` documented in both `tndm ticket create` and `tndm ticket update` option tables
- [ ] Effort default noted in the "Defaults when flags are omitted" line
- [ ] New `Effort` section in the Field Enum Reference table
- [ ] Ticket File Structure note updated to mention `effort` in `meta.toml`
- [ ] `plugin.json` version bumped

**Verify:** `grep -n "effort" plugins/tndm/skills/ticket/references/command-reference.md | wc -l` → at least 6 matches

**Steps:**

- [ ] **Step 1: Add --effort to tndm ticket create options**

In `plugins/tndm/skills/ticket/references/command-reference.md`, find the options block under `tndm ticket create` and add `-e, --effort <SIZE>`:

```
  -e, --effort <SIZE>        Effort estimate. Values: xs | s | m | l | xl
```

Place it after the `-d, --depends-on` line.

Also update the defaults line from:
```
Defaults when flags are omitted: status=todo, priority=p2, type=task, tags=[], depends_on=[].
```
to:
```
Defaults when flags are omitted: status=todo, priority=p2, type=task, effort=unset, tags=[], depends_on=[].
```

- [ ] **Step 2: Add --effort to tndm ticket update options**

In the options block under `tndm ticket update`, add after `-d, --depends-on`:

```
  -e, --effort <SIZE>       Effort estimate. Values: xs | s | m | l | xl
```

- [ ] **Step 3: Add Effort section to Field Enum Reference**

After the `### Type` table, add:

```markdown
### Effort (`--effort`)

| Value | Meaning           |
|-------|-------------------|
| `xs`  | Extra-small       |
| `s`   | Small             |
| `m`   | Medium            |
| `l`   | Large             |
| `xl`  | Extra-large       |

Effort is optional. Omit to leave unset; omitting on update leaves the existing value unchanged.
```

- [ ] **Step 4: Update Ticket File Structure note**

Find the meta.toml comment line:
```
├── meta.toml     # stable metadata: id, title, type, priority, tags, depends_on
```

Replace with:
```
├── meta.toml     # stable metadata: id, title, type, priority, effort (optional), tags, depends_on
```

- [ ] **Step 5: Bump plugin version**

In `plugins/tndm/.claude-plugin/plugin.json`, increment the `version` field by one patch version.

In `plugins/tndm/.codex-plugin/plugin.json`, set the same version.

- [ ] **Step 6: Commit**

```bash
git add plugins/tndm/skills/ticket/references/command-reference.md \
        plugins/tndm/.claude-plugin/plugin.json \
        plugins/tndm/.codex-plugin/plugin.json
git commit -m "docs(plugin): document --effort flag in tndm command reference"
```

```json:metadata
{"files": ["plugins/tndm/skills/ticket/references/command-reference.md", "plugins/tndm/.claude-plugin/plugin.json", "plugins/tndm/.codex-plugin/plugin.json"], "verifyCommand": "grep -n 'effort' plugins/tndm/skills/ticket/references/command-reference.md | wc -l", "acceptanceCriteria": ["--effort documented in create and update", "Effort enum table added", "plugin.json version bumped"], "requiresUserVerification": false}
```
