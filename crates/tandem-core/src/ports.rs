use crate::ticket::{NewTicket, Ticket, TicketId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketChange {
    pub ticket_id: TicketId,
    pub summary: String,
}

pub trait TicketStore {
    type Error;

    fn create_ticket(&self, ticket: NewTicket) -> Result<Ticket, Self::Error>;
    fn load_ticket(&self, id: &TicketId) -> Result<Ticket, Self::Error>;
    fn list_ticket_ids(&self) -> Result<Vec<TicketId>, Self::Error>;
    fn ticket_exists(&self, id: &TicketId) -> Result<bool, Self::Error>;
}

pub trait RepoContext {
    type Error;

    fn current_branch(&self) -> Result<String, Self::Error>;
    fn list_worktrees(&self) -> Result<Vec<String>, Self::Error>;
}

pub trait AwarenessProvider {
    type Error;

    fn collect_ticket_changes(
        &self,
        ticket_id: &TicketId,
    ) -> Result<Vec<TicketChange>, Self::Error>;
}
