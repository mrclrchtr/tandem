use crate::error::ValidationError;
use crate::ticket::TicketEffort;
use crate::ticket::TicketId;
use crate::ticket::TicketPriority;
use crate::ticket::TicketType;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ticket::{TicketEffort, TicketId};

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
}
