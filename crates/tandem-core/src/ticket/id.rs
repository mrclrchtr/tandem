use std::fmt;

use crate::error::ValidationError;

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

#[cfg(test)]
mod tests {
    use super::TicketId;

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
    fn ticket_id_serializes_as_plain_string() {
        let id = TicketId::parse("TNDM-ABC123").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"TNDM-ABC123\"");
    }
}
