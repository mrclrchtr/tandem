use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::error::ValidationError;

/// Generates a string-backed enum with `parse()`, `as_str()`, `FromStr`, `Display`,
/// and `Serialize` from a variant-to-string mapping.
///
/// Usage:
/// ```ignore
/// string_enum! {
///     #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
///     pub enum Foo {
///         #[default]
///         Bar => "bar",
///         Baz => "baz",
///     }
///     error = "invalid foo [possible values: bar, baz]"
/// }
/// ```
macro_rules! string_enum {
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$var_attr:meta])*
                $variant:ident => $str:literal
            ),+ $(,)?
        }
        error = $err:expr
    ) => {
        $(#[$attr])*
        #[derive(serde::Serialize)]
        #[serde(rename_all = "snake_case")]
        $vis enum $name {
            $(
                $(#[$var_attr])*
                $variant,
            )+
        }

        impl $name {
            $vis fn parse(value: &str) -> Result<Self, ValidationError> {
                match value.trim().to_ascii_lowercase().as_str() {
                    $($str => Ok(Self::$variant),)+
                    _ => Err(ValidationError::new($err)),
                }
            }

            $vis fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $str,)+
                }
            }
        }

        impl FromStr for $name {
            type Err = ValidationError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::parse(s)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }


    };
}

/// A validated ticket identifier with no whitespace or path separators.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TicketId(String);

impl TicketId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::new("ticket id must not be empty"));
        }

        if trimmed.chars().any(|character| {
            character.is_whitespace()
                || character.is_control()
                || character == '/'
                || character == '\\'
        }) {
            return Err(ValidationError::new(
                "ticket id must not contain whitespace, control characters, or path separators",
            ));
        }

        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TicketId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl serde::Serialize for TicketId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum TicketType {
        #[default]
        Task => "task",
        Bug => "bug",
        Feature => "feature",
        Chore => "chore",
        Epic => "epic",
    }
    error = "invalid ticket type [possible values: task, bug, feature, chore, epic]"
}

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub enum TicketPriority {
        P0 => "p0",
        P1 => "p1",
        #[default]
        P2 => "p2",
        P3 => "p3",
        P4 => "p4",
    }
    error = "invalid ticket priority [possible values: p0, p1, p2, p3, p4]"
}

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum TicketStatus {
        #[default]
        Todo => "todo",
        InProgress => "in_progress",
        Blocked => "blocked",
        Done => "done",
    }
    error = "invalid ticket status [possible values: todo, in_progress, blocked, done]"
}

string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TicketEffort {
        Xs => "xs",
        S => "s",
        M => "m",
        L => "l",
        Xl => "xl",
    }
    error = "invalid ticket effort [possible values: xs, s, m, l, xl]"
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TicketDocument {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TicketMeta {
    #[serde(skip)]
    pub schema_version: u8,
    pub id: TicketId,
    pub title: String,
    #[serde(rename = "type")]
    pub ticket_type: TicketType,
    pub priority: TicketPriority,
    pub effort: Option<TicketEffort>,
    pub depends_on: Vec<TicketId>,
    pub tags: Vec<String>,
    pub documents: Vec<TicketDocument>,
}

impl TicketMeta {
    pub fn new(id: TicketId, title: impl Into<String>) -> Result<Self, ValidationError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(ValidationError::new("ticket title must not be empty"));
        }

        Ok(Self {
            schema_version: 1,
            id,
            title,
            ticket_type: TicketType::default(),
            priority: TicketPriority::default(),
            effort: None,
            depends_on: Vec::new(),
            tags: Vec::new(),
            documents: vec![TicketDocument {
                name: "content".to_string(),
                path: "content.md".to_string(),
            }],
        })
    }

    pub fn to_canonical_toml(&self) -> String {
        let mut sorted = self.clone();
        sorted.documents.sort_by(|a, b| a.name.cmp(&b.name));
        let body = toml::to_string(&sorted).expect("TicketMeta serialization should not fail");
        format!("schema_version = 1\n{body}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TicketState {
    #[serde(skip)]
    pub schema_version: u8,
    pub status: TicketStatus,
    pub updated_at: String,
    pub revision: u64,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub document_fingerprints: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTicket {
    pub meta: TicketMeta,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticket {
    pub meta: TicketMeta,
    pub state: TicketState,
    pub content: String,
}

impl TicketState {
    pub fn initial(updated_at: impl Into<String>) -> Result<Self, ValidationError> {
        Self::new(updated_at, 1)
    }

    pub fn new(updated_at: impl Into<String>, revision: u64) -> Result<Self, ValidationError> {
        let updated_at = updated_at.into();
        if updated_at.trim().is_empty() {
            return Err(ValidationError::new("ticket updated_at must not be empty"));
        }
        if OffsetDateTime::parse(&updated_at, &Rfc3339).is_err() {
            return Err(ValidationError::new(
                "ticket updated_at must be a valid RFC3339 timestamp",
            ));
        }
        if revision == 0 {
            return Err(ValidationError::new("ticket revision must be >= 1"));
        }

        Ok(Self {
            schema_version: 1,
            status: TicketStatus::default(),
            updated_at,
            revision,
            document_fingerprints: BTreeMap::new(),
        })
    }

    pub fn to_canonical_toml(&self) -> String {
        let body = toml::to_string(&self).expect("TicketState serialization should not fail");
        format!("schema_version = 1\n{body}")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TicketDocument, TicketEffort, TicketId, TicketMeta, TicketPriority, TicketState,
        TicketStatus, TicketType,
    };

    #[test]
    fn parse_accepts_simple_id() {
        let id = TicketId::parse("TICKET-123").expect("TicketId should parse");
        assert_eq!(id.as_str(), "TICKET-123");
    }

    #[test]
    fn parse_trims_whitespace() {
        let id = TicketId::parse("  foo  ").expect("TicketId should parse");
        assert_eq!(id.as_str(), "foo");
    }

    #[test]
    fn parse_rejects_internal_whitespace() {
        let error = TicketId::parse("foo bar").expect_err("TicketId should be rejected");
        assert!(error.message().contains("must not contain"));
    }

    #[test]
    fn parse_rejects_path_separators() {
        let _ = TicketId::parse("foo/bar").expect_err("TicketId should be rejected");
        let _ = TicketId::parse("foo\\bar").expect_err("TicketId should be rejected");
    }

    #[test]
    fn parse_rejects_control_characters() {
        let _ = TicketId::parse("\u{0000}foo").expect_err("TicketId should be rejected");
    }

    #[test]
    fn ticket_type_parse_and_as_str_roundtrip() {
        assert_eq!(
            TicketType::parse("task")
                .expect("type should parse")
                .as_str(),
            "task"
        );
        assert_eq!(
            TicketType::parse("bug")
                .expect("type should parse")
                .as_str(),
            "bug"
        );
        assert_eq!(
            TicketType::parse("feature")
                .expect("type should parse")
                .as_str(),
            "feature"
        );
        assert_eq!(
            TicketType::parse("chore")
                .expect("type should parse")
                .as_str(),
            "chore"
        );
        assert_eq!(
            TicketType::parse("epic")
                .expect("type should parse")
                .as_str(),
            "epic"
        );
        assert_eq!(TicketType::default().as_str(), "task");
    }

    #[test]
    fn ticket_priority_parse_and_as_str_roundtrip() {
        for value in ["p0", "p1", "p2", "p3", "p4"] {
            assert_eq!(
                TicketPriority::parse(value)
                    .expect("priority should parse")
                    .as_str(),
                value
            );
        }
        assert_eq!(TicketPriority::default().as_str(), "p2");
    }

    #[test]
    fn ticket_status_parse_and_as_str_roundtrip() {
        for value in ["todo", "in_progress", "blocked", "done"] {
            assert_eq!(
                TicketStatus::parse(value)
                    .expect("status should parse")
                    .as_str(),
                value
            );
        }
        assert_eq!(TicketStatus::default().as_str(), "todo");
    }

    #[test]
    fn meta_formats_as_canonical_toml() {
        let id = TicketId::parse("TNDM-4K7D9Q").expect("id should parse");
        let meta = TicketMeta::new(id, "Add foo").expect("meta should be valid");

        assert_eq!(
            meta.to_canonical_toml(),
            concat!(
                "schema_version = 1\n",
                "id = \"TNDM-4K7D9Q\"\n",
                "title = \"Add foo\"\n",
                "type = \"task\"\n",
                "priority = \"p2\"\n",
                "depends_on = []\n",
                "tags = []\n",
                "\n",
                "[[documents]]\n",
                "name = \"content\"\n",
                "path = \"content.md\"\n",
            )
        );
    }

    #[test]
    fn state_formats_as_canonical_toml() {
        let state = TicketState::new("2026-03-03T10:00:00Z", 1).expect("state should be valid");

        assert_eq!(
            state.to_canonical_toml(),
            concat!(
                "schema_version = 1\n",
                "status = \"todo\"\n",
                "updated_at = \"2026-03-03T10:00:00Z\"\n",
                "revision = 1\n",
            )
        );
    }

    #[test]
    fn state_accepts_rfc3339_updated_at() {
        let state = TicketState::new("2026-03-03T10:00:00Z", 1)
            .expect("state with RFC3339 timestamp should be valid");

        assert_eq!(state.updated_at, "2026-03-03T10:00:00Z");
        assert_eq!(state.status, TicketStatus::Todo);
        assert_eq!(state.revision, 1);
    }

    #[test]
    fn state_rejects_invalid_updated_at() {
        let error = TicketState::new("not-a-timestamp", 1)
            .expect_err("state with invalid timestamp should be rejected");
        assert_eq!(
            error.message(),
            "ticket updated_at must be a valid RFC3339 timestamp"
        );
    }

    #[test]
    fn state_rejects_empty_updated_at() {
        let error =
            TicketState::new("   ", 1).expect_err("state with empty updated_at should be rejected");
        assert_eq!(error.message(), "ticket updated_at must not be empty");
    }

    #[test]
    fn state_initial_sets_todo_and_revision_one() {
        let state =
            TicketState::initial("2026-03-03T10:00:00Z").expect("initial state should be valid");

        assert_eq!(state.status, TicketStatus::Todo);
        assert_eq!(state.revision, 1);
    }

    #[test]
    fn state_rejects_revision_zero() {
        let error = TicketState::new("2026-03-03T10:00:00Z", 0)
            .expect_err("state with zero revision should be rejected");
        assert_eq!(error.message(), "ticket revision must be >= 1");
    }

    #[test]
    fn ticket_id_serializes_as_plain_string() {
        let id = TicketId::parse("TNDM-ABC123").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"TNDM-ABC123\"");
    }

    #[test]
    #[test]
    fn ticket_meta_serializes_with_type_renamed() {
        let id = TicketId::parse("TNDM-TEST01").unwrap();
        let meta = TicketMeta::new(id, "Test title").unwrap();
        let json: serde_json::Value = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["id"], "TNDM-TEST01");
        assert_eq!(json["title"], "Test title");
        assert_eq!(json["type"], "task");
        assert_eq!(json["priority"], "p2");
        assert!(
            json.get("ticket_type").is_none(),
            "ticket_type should be renamed to type"
        );
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
    fn meta_without_effort_canonical_toml_unchanged() {
        let id = TicketId::parse("TNDM-4K7D9Q").expect("id should parse");
        let meta = TicketMeta::new(id, "Add foo").expect("meta should be valid");

        assert_eq!(
            meta.to_canonical_toml(),
            concat!(
                "schema_version = 1\n",
                "id = \"TNDM-4K7D9Q\"\n",
                "title = \"Add foo\"\n",
                "type = \"task\"\n",
                "priority = \"p2\"\n",
                "depends_on = []\n",
                "tags = []\n",
                "\n",
                "[[documents]]\n",
                "name = \"content\"\n",
                "path = \"content.md\"\n",
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
                "type = \"task\"\n",
                "priority = \"p2\"\n",
                "effort = \"m\"\n",
                "depends_on = []\n",
                "tags = []\n",
                "\n",
                "[[documents]]\n",
                "name = \"content\"\n",
                "path = \"content.md\"\n",
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

    // ─── Document registry tests ─────────────────────────────────

    #[test]
    fn meta_new_registers_default_content_document() {
        let id = TicketId::parse("TNDM-DOC01").expect("id should parse");
        let meta = TicketMeta::new(id, "Docs test").expect("meta should be valid");

        assert_eq!(meta.documents.len(), 1);
        assert_eq!(meta.documents[0].name, "content");
        assert_eq!(meta.documents[0].path, "content.md");
    }

    #[test]
    fn meta_canonical_toml_includes_documents() {
        let id = TicketId::parse("TNDM-DOC02").expect("id should parse");
        let mut meta = TicketMeta::new(id, "Docs toml").expect("meta should be valid");
        // Add a second document to verify sorting
        meta.documents.push(TicketDocument {
            name: "plan".to_string(),
            path: "docs/plan.md".to_string(),
        });

        let toml = meta.to_canonical_toml();
        assert!(
            toml.contains("[[documents]]"),
            "toml should contain [[documents]]: {toml}"
        );
        assert!(
            toml.contains(r#"name = "content""#),
            "toml should contain content doc: {toml}"
        );
        assert!(
            toml.contains(r#"name = "plan""#),
            "toml should contain plan doc: {toml}"
        );
        // Documents should be sorted alphabetically
        let content_pos = toml.find(r#"name = "content""#).unwrap();
        let plan_pos = toml.find(r#"name = "plan""#).unwrap();
        assert!(
            content_pos < plan_pos,
            "content doc should come before plan doc: {toml}"
        );
    }

    #[test]
    fn state_canonical_toml_includes_document_fingerprints() {
        let mut state = TicketState::new("2026-03-03T10:00:00Z", 1).expect("state should be valid");
        state
            .document_fingerprints
            .insert("content".to_string(), "sha256:abc123".to_string());
        state
            .document_fingerprints
            .insert("plan".to_string(), "sha256:def456".to_string());

        let toml = state.to_canonical_toml();
        assert!(
            toml.contains("[document_fingerprints]"),
            "toml should contain [document_fingerprints]: {toml}"
        );
        assert!(
            toml.contains(r#"content = "sha256:abc123""#),
            "toml should contain content fingerprint: {toml}"
        );
        assert!(
            toml.contains(r#"plan = "sha256:def456""#),
            "toml should contain plan fingerprint: {toml}"
        );
    }

    #[test]
    fn state_canonical_toml_omits_fingerprints_when_empty() {
        let state = TicketState::new("2026-03-03T10:00:00Z", 1).expect("state should be valid");

        let toml = state.to_canonical_toml();
        assert!(
            !toml.contains("[document_fingerprints]"),
            "toml should NOT contain [document_fingerprints] when empty: {toml}"
        );
    }

    #[test]
    fn documents_are_sorted_by_name_in_meta() {
        let id = TicketId::parse("TNDM-DOC03").expect("id should parse");
        let mut meta = TicketMeta::new(id, "Sorted docs").expect("meta should be valid");
        // The default "content" doc is already there
        meta.documents.push(TicketDocument {
            name: "alpha".to_string(),
            path: "alpha.md".to_string(),
        });
        meta.documents.push(TicketDocument {
            name: "zeta".to_string(),
            path: "zeta.md".to_string(),
        });
        meta.documents.push(TicketDocument {
            name: "beta".to_string(),
            path: "beta.md".to_string(),
        });

        // Canonical output should have documents sorted by name
        let toml = meta.to_canonical_toml();
        let alpha_pos = toml.find(r#"name = "alpha""#).unwrap();
        let beta_pos = toml.find(r#"name = "beta""#).unwrap();
        let content_pos = toml.find(r#"name = "content""#).unwrap();
        let zeta_pos = toml.find(r#"name = "zeta""#).unwrap();
        assert!(alpha_pos < beta_pos, "alpha should come before beta");
        assert!(beta_pos < content_pos, "beta should come before content");
        assert!(content_pos < zeta_pos, "content should come before zeta");
    }

    /// Tests that `string_enum!` generates correct parsing, case insensitivity,
    /// error messages, Display, and Serialize for the macro-generated enums.
    #[test]
    fn macro_generated_impls() {
        // --- roundtrip for all enums (exercises parse + as_str) ---
        assert_eq!(TicketType::parse("task").unwrap(), TicketType::Task);
        assert_eq!(TicketType::parse("feature").unwrap(), TicketType::Feature);
        assert_eq!(TicketPriority::parse("p0").unwrap(), TicketPriority::P0);
        assert_eq!(TicketPriority::parse("p3").unwrap(), TicketPriority::P3);
        assert_eq!(TicketStatus::parse("todo").unwrap(), TicketStatus::Todo);
        assert_eq!(
            TicketStatus::parse("in_progress").unwrap(),
            TicketStatus::InProgress
        );
        assert_eq!(TicketEffort::parse("xs").unwrap(), TicketEffort::Xs);
        assert_eq!(TicketEffort::parse("xl").unwrap(), TicketEffort::Xl);

        // --- case insensitivity for all enums ---
        assert_eq!(TicketType::parse("BUG").unwrap(), TicketType::Bug);
        assert_eq!(TicketType::parse("FeAtUrE").unwrap(), TicketType::Feature);
        assert_eq!(TicketPriority::parse("P0").unwrap(), TicketPriority::P0);
        assert_eq!(TicketPriority::parse("p4").unwrap(), TicketPriority::P4);
        assert_eq!(TicketStatus::parse("TODO").unwrap(), TicketStatus::Todo);
        assert_eq!(
            TicketStatus::parse("Blocked").unwrap(),
            TicketStatus::Blocked
        );
        assert_eq!(TicketEffort::parse("S").unwrap(), TicketEffort::S);
        assert_eq!(TicketEffort::parse("Xl").unwrap(), TicketEffort::Xl);

        // --- rejects unknown values ---
        let type_error = TicketType::parse("unknown").expect_err("type should be rejected");
        assert_eq!(
            type_error.message(),
            "invalid ticket type [possible values: task, bug, feature, chore, epic]"
        );
        let priority_error = TicketPriority::parse("p9").expect_err("priority should be rejected");
        assert_eq!(
            priority_error.message(),
            "invalid ticket priority [possible values: p0, p1, p2, p3, p4]"
        );
        let status_error = TicketStatus::parse("started").expect_err("status should be rejected");
        assert_eq!(
            status_error.message(),
            "invalid ticket status [possible values: todo, in_progress, blocked, done]"
        );
        let effort_error = TicketEffort::parse("huge").expect_err("effort should be rejected");
        assert_eq!(
            effort_error.message(),
            "invalid ticket effort [possible values: xs, s, m, l, xl]"
        );

        // --- Display renders lower-case string ---
        assert_eq!(format!("{}", TicketType::Bug), "bug");
        assert_eq!(format!("{}", TicketPriority::P1), "p1");
        assert_eq!(format!("{}", TicketStatus::Done), "done");
        assert_eq!(format!("{}", TicketEffort::M), "m");

        // --- Serde JSON serializes as lower-case string ---
        assert_eq!(
            serde_json::to_string(&TicketType::Task).unwrap(),
            "\"task\""
        );
        assert_eq!(
            serde_json::to_string(&TicketPriority::P2).unwrap(),
            "\"p2\""
        );
        assert_eq!(
            serde_json::to_string(&TicketStatus::Blocked).unwrap(),
            "\"blocked\""
        );
        assert_eq!(serde_json::to_string(&TicketEffort::Xl).unwrap(), "\"xl\"");
    }
}
