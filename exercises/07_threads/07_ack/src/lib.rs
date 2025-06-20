use crate::store::TicketId;
use crate::store::TicketStore;
use data::Ticket;
use data::TicketDraft;
use std::sync::mpsc::{Receiver, Sender};

pub mod data;
pub mod store;

// Refer to the tests to understand the expected schema.
pub enum Command {
    Insert {
        draft: TicketDraft,
        response_sender: Sender<TicketId>,
    },
    Get {
        id: TicketId,
        response_sender: Sender<Option<Ticket>>,
    },
}

pub fn launch() -> Sender<Command> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || server(receiver));
    sender
}

// TODO: handle incoming commands as expected.
pub fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();
    loop {
        match receiver.recv() {
            Ok(Command::Insert {
                draft,
                response_sender,
            }) => {
                response_sender.send(store.add_ticket(draft)).unwrap();
            }
            Ok(Command::Get {
                id,
                response_sender,
            }) => match store.get(id) {
                Some(ticket) => response_sender.send(Some(ticket.clone())).unwrap(),
                None => response_sender.send(None).unwrap(),
            },
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break;
            }
        }
    }
}
