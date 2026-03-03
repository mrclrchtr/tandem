use tandem_core::{
    ports::TicketStore,
    ticket::{NewTicket, Ticket, TicketId},
};

#[test]
fn placeholder_uses_tempfile() {
    let _dir = tempfile::tempdir().expect("tempdir");
}

#[test]
fn placeholder_ticketstore_api_exists() {
    let _create: fn(
        &tandem_storage::FileTicketStore,
        NewTicket,
    ) -> Result<Ticket, tandem_storage::StorageError> =
        <tandem_storage::FileTicketStore as TicketStore>::create_ticket;
    let _load: fn(
        &tandem_storage::FileTicketStore,
        &TicketId,
    ) -> Result<Ticket, tandem_storage::StorageError> =
        <tandem_storage::FileTicketStore as TicketStore>::load_ticket;
    let _list: fn(
        &tandem_storage::FileTicketStore,
    ) -> Result<Vec<TicketId>, tandem_storage::StorageError> =
        <tandem_storage::FileTicketStore as TicketStore>::list_ticket_ids;
}
