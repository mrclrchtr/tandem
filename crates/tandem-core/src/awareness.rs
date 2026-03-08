use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::ticket::{Ticket, TicketId};

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
    pub depends_on: Option<AwarenessDependsOnDiff>,
}

impl AwarenessFieldDiffs {
    fn between(current: &Ticket, against: &Ticket) -> Option<Self> {
        let status = diff_value(current.state.status.as_str(), against.state.status.as_str());
        let priority = diff_value(
            current.meta.priority.as_str(),
            against.meta.priority.as_str(),
        );

        let current_depends_on = canonicalize_depends_on(&current.meta.depends_on);
        let against_depends_on = canonicalize_depends_on(&against.meta.depends_on);
        let depends_on =
            (current_depends_on != against_depends_on).then_some(AwarenessDependsOnDiff {
                current: current_depends_on,
                against: against_depends_on,
            });

        let diffs = Self {
            status,
            priority,
            depends_on,
        };

        (!diffs.is_empty()).then_some(diffs)
    }

    fn is_empty(&self) -> bool {
        self.status.is_none() && self.priority.is_none() && self.depends_on.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AwarenessFieldDiff {
    pub current: String,
    pub against: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AwarenessDependsOnDiff {
    pub current: Vec<String>,
    pub against: Vec<String>,
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

#[cfg(test)]
mod tests {
    use crate::ticket::{Ticket, TicketId, TicketMeta, TicketPriority, TicketState, TicketStatus};

    use super::{
        AwarenessChangeKind, AwarenessDependsOnDiff, AwarenessFieldDiff, AwarenessFieldDiffs,
        TicketSnapshot, compare_snapshots,
    };

    #[test]
    fn compare_snapshots_returns_empty_report_for_identical_snapshots() {
        let current = TicketSnapshot::from_tickets([ticket(
            "TNDM-1",
            "Same",
            TicketStatus::Todo,
            TicketPriority::P2,
            &[],
        )]);
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-1",
            "Different content is ignored",
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
            "Current",
            TicketStatus::InProgress,
            TicketPriority::P1,
            &["TNDM-1"],
        )]);
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-3",
            "Against",
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
            Some(AwarenessDependsOnDiff {
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
            "Current",
            TicketStatus::InProgress,
            TicketPriority::P2,
            &["TNDM-1", "TNDM-2"],
        )]);
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-4",
            "Against",
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
            "Current",
            TicketStatus::Todo,
            TicketPriority::P2,
            &["TNDM-2", "TNDM-1"],
        )]);
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-5",
            "Against",
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
            "Current",
            TicketStatus::Todo,
            TicketPriority::P2,
            &["TNDM-2", "TNDM-1", "TNDM-1"],
        )]);
        let against = TicketSnapshot::from_tickets([ticket(
            "TNDM-6",
            "Against",
            TicketStatus::Todo,
            TicketPriority::P2,
            &["TNDM-1", "TNDM-2"],
        )]);

        let report = compare_snapshots("main", &current, &against);

        assert_eq!(report.tickets.len(), 1);
        assert_eq!(report.tickets[0].change, AwarenessChangeKind::Diverged);
        assert_eq!(
            report.tickets[0].fields.depends_on,
            Some(AwarenessDependsOnDiff {
                current: vec![
                    "TNDM-1".to_string(),
                    "TNDM-1".to_string(),
                    "TNDM-2".to_string(),
                ],
                against: vec!["TNDM-1".to_string(), "TNDM-2".to_string()],
            })
        );
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
}
