use log;
use web3::types::H256;

use std::collections::HashMap;
use std::iter::Iterator;

use crate::controller::{Address, Event};

#[derive(Debug)]
pub struct ControllerStorage {
    events: HashMap<H256, Event>,
    events_queue: Vec<Event>,
    events_of_blocked_accounts: HashMap<Address, Vec<Event>>,
}

#[derive(Debug)]
pub enum Error {
    Duplicate,
}

impl ControllerStorage {
    pub fn new() -> Self {
        ControllerStorage {
            events: HashMap::new(),
            events_queue: Vec::new(),
            events_of_blocked_accounts: HashMap::new(),
        }
    }

    pub fn put_event(&mut self, event: &Event) -> Result<(), Error> {
        let message_id = event.message_id();
        match self.events.get(message_id) {
            Some(e) if e == event => Err(Error::Duplicate),
            _ => {
                self.events.insert(*message_id, event.clone());
                Ok(())
            }
        }
    }

    pub fn put_event_to_queue(&mut self, event: Event) {
        self.events_queue.push(event)
    }

    pub fn iter_events_queue(&self) -> impl Iterator<Item = &Event> {
        self.events_queue.iter()
    }

    pub fn clear_events_queue(&mut self) {
        self.events_queue.clear();
    }

    pub fn block_account(&mut self, address: Address) {
        if !self.events_of_blocked_accounts.contains_key(&address) {
            self.events_of_blocked_accounts.insert(address, vec![]);
        } else {
            log::info!("account {:?} is already blocked", address);
        }
    }

    pub fn unblock_account(&mut self, address: Address) {
        match self.events_of_blocked_accounts.get(&address) {
            Some(queue) => {
                let mut queue = queue.to_vec();
                self.events_queue.append(queue.as_mut());
                self.events_of_blocked_accounts.remove(&address);
            }
            None => log::warn!("can not found account queue for {:?}", address),
        }
    }

    pub fn is_account_blocked(&self, address: Option<Address>) -> bool {
        match address {
            None => false,
            Some(a) => self.events_of_blocked_accounts.contains_key(&a),
        }
    }

    pub fn put_event_to_account_queue(&mut self, event: Event) {
        let sender = event
            .sender()
            .expect("called put_event_to_account_queue for invalid event");
        match self.events_of_blocked_accounts.get(&sender) {
            Some(queue) => {
                let mut queue = queue.to_vec();
                queue.push(event);
                self.events_of_blocked_accounts.insert(sender, queue);
            }
            None => log::warn!("can not found account queue for {:?}", sender),
        }
    }
}
