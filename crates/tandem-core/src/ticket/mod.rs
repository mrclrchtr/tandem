use std::fmt;
use std::str::FromStr;

use crate::error::ValidationError;

pub mod id;
pub use id::*;

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

pub mod meta;
pub use meta::*;
pub mod state;
pub use state::*;

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

#[cfg(test)]
mod tests {
    use super::*;

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
