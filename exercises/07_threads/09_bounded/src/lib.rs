// TODO: Convert the implementation to use bounded channels.
use crate::data::{Ticket, TicketDraft};
use crate::store::{TicketId, TicketStore};
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender};
use std::sync::mpsc::{SyncSender, TrySendError};

pub mod data;
pub mod store;

#[derive(Clone)]
pub struct TicketStoreClient {
    sender: SyncSender<Command>,
}

impl TicketStoreClient {
    pub fn insert(&self, draft: TicketDraft) -> Result<TicketId, TrySendError<String>> {
        let (server_sender, server_receiver) = channel();
        match self.sender.try_send(Command::Insert {
            draft,
            response_channel: server_sender,
        }) {
            Ok(()) => Ok(server_receiver.recv().unwrap()),
            Err(_) => Err(TrySendError::Full("Channel Full".to_string())),
        }
    }

    pub fn get(&self, id: TicketId) -> Result<Option<Ticket>, TrySendError<String>> {
        let (server_sender, server_receiver) = channel();
        match self.sender.try_send(Command::Get {
            id,
            response_channel: server_sender,
        }) {
            Ok(()) => Ok(server_receiver.recv().unwrap()),
            Err(_) => Err(TrySendError::Full("Channel Full".to_string())),
        }
    }
}

pub fn launch(capacity: usize) -> TicketStoreClient {
    let (sender, receiver) = sync_channel(capacity);
    std::thread::spawn(move || server(receiver));
    TicketStoreClient { sender }
}

enum Command {
    Insert {
        draft: TicketDraft,
        response_channel: Sender<TicketId>,
    },
    Get {
        id: TicketId,
        response_channel: Sender<Option<Ticket>>,
    },
}

pub fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();
    loop {
        match receiver.recv() {
            Ok(Command::Insert {
                draft,
                response_channel,
            }) => {
                let id = store.add_ticket(draft);
                response_channel.send(id).unwrap()
            }
            Ok(Command::Get {
                id,
                response_channel,
            }) => {
                let ticket = store.get(id);
                match ticket {
                    Some(ticket) => response_channel.send(Some(ticket.clone())).unwrap(),
                    None => response_channel.send(None).unwrap(),
                }
            }
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break;
            }
        }
    }
}
