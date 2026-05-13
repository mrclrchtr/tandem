use std::collections::BTreeMap;

use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::error::ValidationError;
use crate::ticket::TicketStatus;

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
    use super::*;
    use crate::ticket::TicketStatus;

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
    fn ticket_state_serializes_all_fields() {
        let state = TicketState::new("2026-03-17T12:00:00Z", 3).unwrap();
        let json: serde_json::Value = serde_json::to_value(&state).unwrap();
        assert_eq!(json["status"], "todo");
        assert_eq!(json["updated_at"], "2026-03-17T12:00:00Z");
        assert_eq!(json["revision"], 3);
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
}
