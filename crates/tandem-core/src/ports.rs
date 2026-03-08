use crate::{
    awareness::TicketSnapshot,
    ticket::{NewTicket, Ticket, TicketId},
};

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

pub trait AwarenessSnapshotProvider {
    type Error;

    fn load_current_snapshot(&self) -> Result<TicketSnapshot, Self::Error>;
    fn load_snapshot_for_ref(&self, reference: &str) -> Result<TicketSnapshot, Self::Error>;
}
