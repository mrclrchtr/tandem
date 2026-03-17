use std::fmt;

use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::error::ValidationError;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TicketType {
    #[default]
    Task,
    Bug,
    Feature,
    Chore,
    Epic,
}

impl TicketType {
    pub fn parse(value: &str) -> Result<Self, ValidationError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "task" => Ok(Self::Task),
            "bug" => Ok(Self::Bug),
            "feature" => Ok(Self::Feature),
            "chore" => Ok(Self::Chore),
            "epic" => Ok(Self::Epic),
            _ => Err(ValidationError::new("invalid ticket type")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Bug => "bug",
            Self::Feature => "feature",
            Self::Chore => "chore",
            Self::Epic => "epic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TicketPriority {
    P0,
    P1,
    #[default]
    P2,
    P3,
    P4,
}

impl TicketPriority {
    pub fn parse(value: &str) -> Result<Self, ValidationError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "p0" => Ok(Self::P0),
            "p1" => Ok(Self::P1),
            "p2" => Ok(Self::P2),
            "p3" => Ok(Self::P3),
            "p4" => Ok(Self::P4),
            _ => Err(ValidationError::new("invalid ticket priority")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::P0 => "p0",
            Self::P1 => "p1",
            Self::P2 => "p2",
            Self::P3 => "p3",
            Self::P4 => "p4",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TicketStatus {
    #[default]
    Todo,
    InProgress,
    Blocked,
    Done,
}

impl TicketStatus {
    pub fn parse(value: &str) -> Result<Self, ValidationError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "todo" => Ok(Self::Todo),
            "in_progress" => Ok(Self::InProgress),
            "blocked" => Ok(Self::Blocked),
            "done" => Ok(Self::Done),
            _ => Err(ValidationError::new("invalid ticket status")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Done => "done",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketMeta {
    pub id: TicketId,
    pub title: String,
    pub ticket_type: TicketType,
    pub priority: TicketPriority,
    pub depends_on: Vec<TicketId>,
    pub tags: Vec<String>,
}

impl TicketMeta {
    pub fn new(id: TicketId, title: impl Into<String>) -> Result<Self, ValidationError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(ValidationError::new("ticket title must not be empty"));
        }

        Ok(Self {
            id,
            title,
            ticket_type: TicketType::default(),
            priority: TicketPriority::default(),
            depends_on: Vec::new(),
            tags: Vec::new(),
        })
    }

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketState {
    pub status: TicketStatus,
    pub updated_at: String,
    pub revision: u64,
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
            status: TicketStatus::default(),
            updated_at,
            revision,
        })
    }

    pub fn to_canonical_toml(&self) -> String {
        let mut output = String::new();
        output.push_str("schema_version = 1\n");
        output.push_str("status = ");
        output.push_str(&toml_basic_string(self.status.as_str()));
        output.push('\n');
        output.push_str("updated_at = ");
        output.push_str(&toml_basic_string(&self.updated_at));
        output.push('\n');
        output.push_str("revision = ");
        output.push_str(&self.revision.to_string());
        output.push('\n');
        output
    }
}

fn toml_basic_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

fn toml_string_array<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    let values: Vec<&str> = values.into_iter().collect();
    if values.is_empty() {
        return "[]".to_string();
    }

    let joined = values
        .into_iter()
        .map(toml_basic_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{joined}]")
}

#[cfg(test)]
mod tests {
    use super::{TicketId, TicketMeta, TicketPriority, TicketState, TicketStatus, TicketType};

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
    fn ticket_type_parse_is_case_insensitive() {
        assert_eq!(TicketType::parse("Task").unwrap(), TicketType::Task);
        assert_eq!(TicketType::parse("TASK").unwrap(), TicketType::Task);
        assert_eq!(TicketType::parse("BUG").unwrap(), TicketType::Bug);
        assert_eq!(TicketType::parse("Feature").unwrap(), TicketType::Feature);
        assert_eq!(TicketType::parse("CHORE").unwrap(), TicketType::Chore);
        assert_eq!(TicketType::parse("Epic").unwrap(), TicketType::Epic);
    }

    #[test]
    fn ticket_type_parse_rejects_unknown_value() {
        let error = TicketType::parse("unknown").expect_err("type should be rejected");
        assert_eq!(error.message(), "invalid ticket type");
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
    fn ticket_priority_parse_is_case_insensitive() {
        assert_eq!(TicketPriority::parse("P0").unwrap(), TicketPriority::P0);
        assert_eq!(TicketPriority::parse("P2").unwrap(), TicketPriority::P2);
        assert_eq!(TicketPriority::parse("P4").unwrap(), TicketPriority::P4);
    }

    #[test]
    fn ticket_priority_parse_rejects_unknown_value() {
        let error = TicketPriority::parse("p9").expect_err("priority should be rejected");
        assert_eq!(error.message(), "invalid ticket priority");
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
    fn ticket_status_parse_is_case_insensitive() {
        assert_eq!(TicketStatus::parse("Todo").unwrap(), TicketStatus::Todo);
        assert_eq!(TicketStatus::parse("TODO").unwrap(), TicketStatus::Todo);
        assert_eq!(
            TicketStatus::parse("IN_PROGRESS").unwrap(),
            TicketStatus::InProgress
        );
        assert_eq!(
            TicketStatus::parse("In_Progress").unwrap(),
            TicketStatus::InProgress
        );
        assert_eq!(
            TicketStatus::parse("BLOCKED").unwrap(),
            TicketStatus::Blocked
        );
        assert_eq!(TicketStatus::parse("Done").unwrap(), TicketStatus::Done);
    }

    #[test]
    fn ticket_status_parse_rejects_unknown_value() {
        let error = TicketStatus::parse("started").expect_err("status should be rejected");
        assert_eq!(error.message(), "invalid ticket status");
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
}
