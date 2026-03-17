use crate::ticket::{NewTicket, Ticket, TicketId};

pub trait TicketStore {
    type Error;

    fn create_ticket(&self, ticket: NewTicket) -> Result<Ticket, Self::Error>;
    fn load_ticket(&self, id: &TicketId) -> Result<Ticket, Self::Error>;
    fn list_ticket_ids(&self) -> Result<Vec<TicketId>, Self::Error>;
    fn update_ticket(&self, ticket: &Ticket) -> Result<Ticket, Self::Error>;
    fn ticket_exists(&self, id: &TicketId) -> Result<bool, Self::Error>;
}

pub trait RepoContext {
    type Error;

    fn current_branch(&self) -> Result<String, Self::Error>;
    fn list_worktrees(&self) -> Result<Vec<String>, Self::Error>;
}
