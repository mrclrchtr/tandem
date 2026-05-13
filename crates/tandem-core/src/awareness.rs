use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::ticket::{Ticket, TicketEffort, TicketId};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TicketSnapshot {
    pub tickets: BTreeMap<TicketId, Ticket>,
}

impl TicketSnapshot {
    pub fn from_tickets(tickets: impl IntoIterator<Item = Ticket>) -> Self {
        Self {
            tickets: tickets
                .into_iter()
                .map(|ticket| (ticket.meta.id.clone(), ticket))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AwarenessReport {
    pub schema_version: u64,
    pub against: String,
    pub tickets: Vec<AwarenessTicketChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AwarenessTicketChange {
    pub id: String,
    pub change: AwarenessChangeKind,
    #[serde(default, skip_serializing_if = "AwarenessFieldDiffs::is_empty")]
    pub fields: AwarenessFieldDiffs,
}

impl AwarenessTicketChange {
    fn added_current(id: &str) -> Self {
        Self {
            id: id.to_string(),
            change: AwarenessChangeKind::AddedCurrent,
            fields: AwarenessFieldDiffs::default(),
        }
    }

    fn added_against(id: &str) -> Self {
        Self {
            id: id.to_string(),
            change: AwarenessChangeKind::AddedAgainst,
            fields: AwarenessFieldDiffs::default(),
        }
    }

    fn diverged(id: &str, fields: AwarenessFieldDiffs) -> Self {
        Self {
            id: id.to_string(),
            change: AwarenessChangeKind::Diverged,
            fields,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AwarenessChangeKind {
    AddedCurrent,
    AddedAgainst,
    Diverged,
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documents: Option<Vec<AwarenessDocEntry>>,
}

impl AwarenessFieldDiffs {
    fn between(current: &Ticket, against: &Ticket) -> Option<Self> {
        let status = diff_value(current.state.status.as_str(), against.state.status.as_str());
        let priority = diff_value(
            current.meta.priority.as_str(),
            against.meta.priority.as_str(),
        );
        let effort = diff_value(
            current.meta.effort.map(TicketEffort::as_str).unwrap_or("-"),
            against.meta.effort.map(TicketEffort::as_str).unwrap_or("-"),
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

        // Compare document fingerprint changes — report only changed entries
        let all_names: BTreeSet<&String> = current
            .state
            .document_fingerprints
            .keys()
            .chain(against.state.document_fingerprints.keys())
            .collect();

        let changed_entries: Vec<AwarenessDocEntry> = all_names
            .into_iter()
            .filter_map(|name| {
                let current_fp = current.state.document_fingerprints.get(name);
                let against_fp = against.state.document_fingerprints.get(name);
                if current_fp != against_fp {
                    Some(AwarenessDocEntry {
                        name: name.clone(),
                        current: current_fp.cloned().unwrap_or_default(),
                        against: against_fp.cloned().unwrap_or_default(),
                    })
                } else {
                    None
                }
            })
            .collect();

        let documents = (!changed_entries.is_empty()).then_some(changed_entries);

        let diffs = Self {
            status,
            priority,
            effort,
            title,
            ticket_type,
            depends_on,
            tags,
            documents,
        };

        (!diffs.is_empty()).then_some(diffs)
    }

    fn is_empty(&self) -> bool {
        self.status.is_none()
            && self.priority.is_none()
            && self.effort.is_none()
            && self.title.is_none()
            && self.ticket_type.is_none()
            && self.depends_on.is_none()
            && self.tags.is_none()
            && self.documents.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AwarenessFieldDiff {
    pub current: String,
    pub against: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AwarenessVecDiff {
    pub current: Vec<String>,
    pub against: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AwarenessDocEntry {
    pub name: String,
    pub current: String,
    pub against: String,
}

pub fn compare_snapshots(
    against: impl Into<String>,
    current: &TicketSnapshot,
    against_snapshot: &TicketSnapshot,
) -> AwarenessReport {
    let mut tickets = Vec::new();

    for id in current
        .tickets
        .keys()
        .chain(against_snapshot.tickets.keys())
        .collect::<BTreeSet<_>>()
    {
        match (current.tickets.get(id), against_snapshot.tickets.get(id)) {
            (Some(_), None) => tickets.push(AwarenessTicketChange::added_current(id.as_str())),
            (None, Some(_)) => tickets.push(AwarenessTicketChange::added_against(id.as_str())),
            (Some(current_ticket), Some(against_ticket)) => {
                if let Some(fields) = AwarenessFieldDiffs::between(current_ticket, against_ticket) {
                    tickets.push(AwarenessTicketChange::diverged(id.as_str(), fields));
                }
            }
            (None, None) => unreachable!(),
        }
    }

    AwarenessReport {
        schema_version: 1,
        against: against.into(),
        tickets,
    }
}

fn diff_value(current: &str, against: &str) -> Option<AwarenessFieldDiff> {
    (current != against).then(|| AwarenessFieldDiff {
        current: current.to_string(),
        against: against.to_string(),
    })
}

fn canonicalize_depends_on(depends_on: &[TicketId]) -> Vec<String> {
    let mut canonical = depends_on
        .iter()
        .map(TicketId::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    canonical.sort();
    canonical
}

fn canonicalize_tags(tags: &[String]) -> Vec<String> {
    let mut canonical = tags.to_vec();
    canonical.sort();
    canonical
}

#[cfg(test)]
mod tests {
    use crate::ticket::{
        Ticket, TicketEffort, TicketId, TicketMeta, TicketPriority, TicketState, TicketStatus,
        TicketType,
    };

    use super::{
        AwarenessChangeKind, AwarenessFieldDiff, AwarenessFieldDiffs, AwarenessVecDiff,
        TicketSnapshot, compare_snapshots,
    };

    #[test]
    fn compare_snapshots_returns_empty_report_for_identical_snapshots() {
        let current = TicketSnapshot::from_tickets([ticket(
            "TNDM-1",
            "Same title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        )]);
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-1",
            "Same title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        )]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.schema_version, 1);
        assert_eq!(report.against, "main");
        assert!(report.tickets.is_empty());
    }

    #[test]
    fn compare_snapshots_marks_current_only_tickets_as_added_current() {
        let current = TicketSnapshot::from_tickets([ticket(
            "TNDM-2",
            "Current only",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        )]);
        let against = TicketSnapshot::default();

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        assert_eq!(report.tickets[0].id, "TNDM-2");
        assert_eq!(report.tickets[0].change, AwarenessChangeKind::AddedCurrent);
        assert_eq!(report.tickets[0].fields, AwarenessFieldDiffs::default());
    }

    #[test]
    fn compare_snapshots_marks_against_only_tickets_as_added_against() {
        let current = TicketSnapshot::default();
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-1",
            "Against only",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        )]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        assert_eq!(report.tickets[0].id, "TNDM-1");
        assert_eq!(report.tickets[0].change, AwarenessChangeKind::AddedAgainst);
        assert_eq!(report.tickets[0].fields, AwarenessFieldDiffs::default());
    }

    #[test]
    fn compare_snapshots_reports_diverged_status_priority_and_depends_on() {
        let current = TicketSnapshot::from_tickets([ticket(
            "TNDM-3",
            "Shared title",
            TicketStatus::InProgress,
            TicketPriority::P1,
            &["TNDM-1"],
        )]);
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-3",
            "Shared title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        )]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        assert_eq!(report.tickets[0].id, "TNDM-3");
        assert_eq!(report.tickets[0].change, AwarenessChangeKind::Diverged);

        let fields = &report.tickets[0].fields;
        assert_eq!(
            fields.status,
            Some(AwarenessFieldDiff {
                current: "in_progress".to_string(),
                against: "todo".to_string(),
            })
        );
        assert_eq!(
            fields.priority,
            Some(AwarenessFieldDiff {
                current: "p1".to_string(),
                against: "p2".to_string(),
            })
        );
        assert_eq!(
            fields.depends_on,
            Some(AwarenessVecDiff {
                current: vec!["TNDM-1".to_string()],
                against: Vec::new(),
            })
        );
    }

    #[test]
    fn compare_snapshots_uses_stable_ticket_and_field_ordering() {
        let current = TicketSnapshot::from_tickets([
            ticket(
                "TNDM-2",
                "Current only",
                TicketStatus::Todo,
                TicketPriority::P2,
                &[],
            ),
            ticket(
                "TNDM-3",
                "Diverged",
                TicketStatus::InProgress,
                TicketPriority::P1,
                &["TNDM-1"],
            ),
        ]);
        let against = TicketSnapshot::from_tickets([
            ticket(
                "TNDM-1",
                "Against only",
                TicketStatus::Todo,
                TicketPriority::P2,
                &[],
            ),
            ticket(
                "TNDM-3",
                "Diverged",
                TicketStatus::Todo,
                TicketPriority::P2,
                &[],
            ),
        ]);

        let report = compare_snapshots("main", &current, &against);
        let json = serde_json::to_string(&report).unwrap();

        assert_eq!(
            report
                .tickets
                .iter()
                .map(|ticket| ticket.id.as_str())
                .collect::<Vec<_>>(),
            vec!["TNDM-1", "TNDM-2", "TNDM-3"]
        );
        assert!(json.contains(
            "\"fields\":{\"status\":{\"current\":\"in_progress\",\"against\":\"todo\"},\"priority\":{\"current\":\"p1\",\"against\":\"p2\"},\"depends_on\":{\"current\":[\"TNDM-1\"],\"against\":[]}}"
        ));
    }

    #[test]
    fn compare_snapshots_omits_unchanged_fields_from_serialized_json() {
        let current = TicketSnapshot::from_tickets([ticket(
            "TNDM-4",
            "Shared title",
            TicketStatus::InProgress,
            TicketPriority::P2,
            &["TNDM-1", "TNDM-2"],
        )]);
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-4",
            "Shared title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &["TNDM-2", "TNDM-1"],
        )]);

        let report = compare_snapshots("main", &current, &against);
        let json = serde_json::to_string(&report).unwrap();

        assert_eq!(report.tickets.len(), 1);
        assert!(json.contains("\"status\":{\"current\":\"in_progress\",\"against\":\"todo\"}"));
        assert!(!json.contains("\"priority\":"));
        assert!(!json.contains("\"depends_on\":"));
    }

    #[test]
    fn compare_snapshots_ignores_depends_on_order_when_values_match() {
        let current = TicketSnapshot::from_tickets([ticket(
            "TNDM-5",
            "Shared title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &["TNDM-2", "TNDM-1"],
        )]);
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-5",
            "Shared title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &["TNDM-1", "TNDM-2"],
        )]);

        let report = compare_snapshots("main", &current, &against);

        assert!(report.tickets.is_empty());
    }

    #[test]
    fn compare_snapshots_preserves_duplicate_depends_on_entries_as_difference() {
        let current = TicketSnapshot::from_tickets([ticket(
            "TNDM-6",
            "Shared title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &["TNDM-2", "TNDM-1", "TNDM-1"],
        )]);
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-6",
            "Shared title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &["TNDM-1", "TNDM-2"],
        )]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        assert_eq!(report.tickets[0].change, AwarenessChangeKind::Diverged);
        assert_eq!(
            report.tickets[0].fields.depends_on,
            Some(AwarenessVecDiff {
                current: vec![
                    "TNDM-1".to_string(),
                    "TNDM-1".to_string(),
                    "TNDM-2".to_string(),
                ],
                against: vec!["TNDM-1".to_string(), "TNDM-2".to_string()],
            })
        );
    }

    #[test]
    fn compare_snapshots_reports_diverged_title() {
        let current = TicketSnapshot::from_tickets([ticket(
            "TNDM-1",
            "Old title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        )]);
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-1",
            "New title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        )]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        assert_eq!(report.tickets[0].change, AwarenessChangeKind::Diverged);
        assert_eq!(
            report.tickets[0].fields.title,
            Some(AwarenessFieldDiff {
                current: "Old title".to_string(),
                against: "New title".to_string(),
            })
        );
        assert!(report.tickets[0].fields.status.is_none());
    }

    #[test]
    fn compare_snapshots_reports_diverged_ticket_type() {
        let mut current_ticket = ticket(
            "TNDM-1",
            "Title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        current_ticket.meta.ticket_type = TicketType::Bug;
        let current = TicketSnapshot::from_tickets([current_ticket]);

        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-1",
            "Title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        )]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        assert_eq!(report.tickets[0].change, AwarenessChangeKind::Diverged);
        assert_eq!(
            report.tickets[0].fields.ticket_type,
            Some(AwarenessFieldDiff {
                current: "bug".to_string(),
                against: "task".to_string(),
            })
        );
    }

    #[test]
    fn compare_snapshots_reports_diverged_tags() {
        let mut current_ticket = ticket(
            "TNDM-1",
            "Title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        current_ticket.meta.tags = vec!["auth".to_string(), "backend".to_string()];
        let current = TicketSnapshot::from_tickets([current_ticket]);

        let mut against_ticket = ticket(
            "TNDM-1",
            "Title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        against_ticket.meta.tags = vec!["frontend".to_string()];
        let against = TicketSnapshot::from_tickets([against_ticket]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        assert_eq!(report.tickets[0].change, AwarenessChangeKind::Diverged);
        assert_eq!(
            report.tickets[0].fields.tags,
            Some(AwarenessVecDiff {
                current: vec!["auth".to_string(), "backend".to_string()],
                against: vec!["frontend".to_string()],
            })
        );
    }

    #[test]
    fn compare_snapshots_ignores_tags_order_when_values_match() {
        let mut current_ticket = ticket(
            "TNDM-1",
            "Title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        current_ticket.meta.tags = vec!["b".to_string(), "a".to_string()];
        let current = TicketSnapshot::from_tickets([current_ticket]);

        let mut against_ticket = ticket(
            "TNDM-1",
            "Title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        against_ticket.meta.tags = vec!["a".to_string(), "b".to_string()];
        let against = TicketSnapshot::from_tickets([against_ticket]);

        let report = compare_snapshots("main", &current, &against);

        assert!(report.tickets.is_empty());
    }

    #[test]
    fn compare_snapshots_reports_diverged_effort() {
        let mut current_ticket = ticket(
            "TNDM-1",
            "Title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        current_ticket.meta.effort = Some(TicketEffort::M);
        let current = TicketSnapshot::from_tickets([current_ticket]);

        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-1",
            "Title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        )]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        assert_eq!(report.tickets[0].change, AwarenessChangeKind::Diverged);
        assert_eq!(
            report.tickets[0].fields.effort,
            Some(AwarenessFieldDiff {
                current: "m".to_string(),
                against: "-".to_string(),
            })
        );
    }

    #[test]
    fn compare_snapshots_omits_effort_diff_when_both_unset() {
        let current = TicketSnapshot::from_tickets([ticket(
            "TNDM-1",
            "Title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        )]);
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-1",
            "Title",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        )]);

        let report = compare_snapshots("main", &current, &against);

        assert!(report.tickets.is_empty());
    }

    fn ticket(
        id: &str,
        title: &str,
        status: TicketStatus,
        priority: TicketPriority,
        depends_on: &[&str],
    ) -> Ticket {
        let id = TicketId::parse(id).unwrap();
        let mut meta = TicketMeta::new(id.clone(), title).unwrap();
        meta.priority = priority;
        meta.depends_on = depends_on
            .iter()
            .map(|value| TicketId::parse(*value).unwrap())
            .collect();

        let mut state = TicketState::new("2026-03-08T00:00:00Z", 1).unwrap();
        state.status = status;

        Ticket {
            meta,
            state,
            content: format!("{title}\n"),
        }
    }

    #[test]
    fn compare_snapshots_detects_document_fingerprint_diff() {
        let mut current_ticket = ticket(
            "TNDM-DOCAW",
            "Doc awareness",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        current_ticket
            .state
            .document_fingerprints
            .insert("content".to_string(), "sha256:abc".to_string());

        let mut against_ticket = ticket(
            "TNDM-DOCAW",
            "Doc awareness",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        against_ticket
            .state
            .document_fingerprints
            .insert("content".to_string(), "sha256:def".to_string());

        let current = TicketSnapshot::from_tickets([current_ticket]);
        let against = TicketSnapshot::from_tickets([against_ticket]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        assert_eq!(report.tickets[0].change, AwarenessChangeKind::Diverged);
        let docs = &report.tickets[0]
            .fields
            .documents
            .as_ref()
            .expect("should have document diff");
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].name, "content");
        assert_eq!(docs[0].current, "sha256:abc");
        assert_eq!(docs[0].against, "sha256:def");
    }

    #[test]
    fn compare_snapshots_reports_only_changed_documents() {
        let mut current_ticket = ticket(
            "TNDM-DOC2",
            "Doc partial change",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        current_ticket
            .state
            .document_fingerprints
            .insert("content".to_string(), "sha256:abc".to_string());
        current_ticket
            .state
            .document_fingerprints
            .insert("design".to_string(), "sha256:fff".to_string());

        let mut against_ticket = ticket(
            "TNDM-DOC2",
            "Doc partial change",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        against_ticket
            .state
            .document_fingerprints
            .insert("content".to_string(), "sha256:def".to_string());
        against_ticket
            .state
            .document_fingerprints
            .insert("design".to_string(), "sha256:fff".to_string());

        let current = TicketSnapshot::from_tickets([current_ticket]);
        let against = TicketSnapshot::from_tickets([against_ticket]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        let docs = &report.tickets[0]
            .fields
            .documents
            .as_ref()
            .expect("should have document diff");
        assert_eq!(docs.len(), 1, "only the changed doc should appear");
        assert_eq!(docs[0].name, "content");
        assert_eq!(docs[0].current, "sha256:abc");
        assert_eq!(docs[0].against, "sha256:def");
    }

    #[test]
    fn compare_snapshots_reports_added_document() {
        let mut current_ticket = ticket(
            "TNDM-DOCADD",
            "Doc added",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        current_ticket
            .state
            .document_fingerprints
            .insert("new_doc".to_string(), "sha256:abc".to_string());

        let against_ticket = ticket(
            "TNDM-DOCADD",
            "Doc added",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );

        let current = TicketSnapshot::from_tickets([current_ticket]);
        let against = TicketSnapshot::from_tickets([against_ticket]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        let docs = &report.tickets[0]
            .fields
            .documents
            .as_ref()
            .expect("should have document diff");
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].name, "new_doc");
        assert_eq!(docs[0].current, "sha256:abc");
        assert_eq!(
            docs[0].against, "",
            "added doc has no fingerprint in against"
        );
    }

    #[test]
    fn compare_snapshots_reports_removed_document() {
        let current_ticket = ticket(
            "TNDM-DOCRM",
            "Doc removed",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );

        let mut against_ticket = ticket(
            "TNDM-DOCRM",
            "Doc removed",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        against_ticket
            .state
            .document_fingerprints
            .insert("removed_doc".to_string(), "sha256:def".to_string());

        let current = TicketSnapshot::from_tickets([current_ticket]);
        let against = TicketSnapshot::from_tickets([against_ticket]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        let docs = &report.tickets[0]
            .fields
            .documents
            .as_ref()
            .expect("should have document diff");
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].name, "removed_doc");
        assert_eq!(
            docs[0].current, "",
            "removed doc has no fingerprint in current"
        );
        assert_eq!(docs[0].against, "sha256:def");
    }

    #[test]
    fn compare_snapshots_omits_documents_when_fingerprints_unchanged() {
        let mut current_ticket = ticket(
            "TNDM-DOCID",
            "Doc identical",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        current_ticket
            .state
            .document_fingerprints
            .insert("content".to_string(), "sha256:abc".to_string());

        let mut against_ticket = ticket(
            "TNDM-DOCID",
            "Doc identical",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        );
        against_ticket
            .state
            .document_fingerprints
            .insert("content".to_string(), "sha256:abc".to_string());

        let current = TicketSnapshot::from_tickets([current_ticket]);
        let against = TicketSnapshot::from_tickets([against_ticket]);

        let report = compare_snapshots("main", &current, &against);

        assert!(report.tickets.is_empty(), "no diff when fingerprints match");
    }
}
