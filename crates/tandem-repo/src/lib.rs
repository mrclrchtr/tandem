use std::fmt;

use tandem_core::{
    ports::{AwarenessProvider, RepoContext, TicketChange},
    ticket::TicketId,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct GitRepoContext;

#[derive(Debug, Default, Clone, Copy)]
pub struct GitAwarenessProvider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoError {
    message: String,
}

impl RepoError {
    fn not_implemented(operation: &str) -> Self {
        Self {
            message: format!("repo operation `{operation}` is not implemented"),
        }
    }
}

impl fmt::Display for RepoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for RepoError {}

impl RepoContext for GitRepoContext {
    type Error = RepoError;

    fn current_branch(&self) -> Result<String, Self::Error> {
        Err(RepoError::not_implemented("current_branch"))
    }

    fn list_worktrees(&self) -> Result<Vec<String>, Self::Error> {
        Err(RepoError::not_implemented("list_worktrees"))
    }
}

impl AwarenessProvider for GitAwarenessProvider {
    type Error = RepoError;

    fn collect_ticket_changes(
        &self,
        _ticket_id: &TicketId,
    ) -> Result<Vec<TicketChange>, Self::Error> {
        Err(RepoError::not_implemented("collect_ticket_changes"))
    }
}
