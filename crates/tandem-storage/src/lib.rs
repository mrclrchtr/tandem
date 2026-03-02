use std::fmt;

use tandem_core::{ports::TicketStore, ticket::TicketId};

#[derive(Debug, Default, Clone, Copy)]
pub struct FileTicketStore;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageError {
    message: String,
}

impl StorageError {
    fn not_implemented(operation: &str) -> Self {
        Self {
            message: format!("storage operation `{operation}` is not implemented"),
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for StorageError {}

impl TicketStore for FileTicketStore {
    type Error = StorageError;

    fn create_ticket(&self, _id: &TicketId) -> Result<(), Self::Error> {
        Err(StorageError::not_implemented("create_ticket"))
    }

    fn list_tickets(&self) -> Result<Vec<TicketId>, Self::Error> {
        Err(StorageError::not_implemented("list_tickets"))
    }

    fn ticket_exists(&self, _id: &TicketId) -> Result<bool, Self::Error> {
        Err(StorageError::not_implemented("ticket_exists"))
    }
}
