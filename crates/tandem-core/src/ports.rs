use std::path::Path;

use crate::ticket::{NewTicket, Ticket, TicketId};

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

pub trait MaterializedRefSnapshot {
    fn path(&self) -> &Path;
}

pub trait AwarenessRefMaterializer {
    type Error;
    type Snapshot: MaterializedRefSnapshot;

    fn materialize_ref_snapshot(
        &self,
        reference: &str,
    ) -> Result<Option<Self::Snapshot>, Self::Error>;
}
