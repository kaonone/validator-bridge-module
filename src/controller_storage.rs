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

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use web3::types::{H160, U256};

    const MESSAGE_ID: [u8; 32] = [0; 32];
    const MESSAGE_ID2: [u8; 32] = [1; 32];
    const ETH_ADDRESS: [u8; 20] = [7; 20];
    const SUB_ADDRESS: [u8; 32] = [7; 32];
    const TOKEN_ID: u64 = 0;
    const AMOUNT: u128 = 0;
    const BLOCK_NUMBER: u128 = 0;

    #[test]
    fn put_event_tests() {
        let mut storage = ControllerStorage::new();
        let event = Event::EthBridgePausedMessage(H256::from_slice(&MESSAGE_ID), BLOCK_NUMBER);
        assert_eq!(Ok(()), storage.put_event(&event));
        assert_eq!(Err(Error::Duplicate), storage.put_event(&event));
    }

    #[test]
    fn event_queue_tests() {
        let mut storage = ControllerStorage::new();
        let event = Event::EthBridgePausedMessage(H256::from_slice(&MESSAGE_ID), BLOCK_NUMBER);
        let event2 = Event::EthBridgePausedMessage(H256::from_slice(&MESSAGE_ID2), BLOCK_NUMBER);

        let empty_vec: Vec<Event> = vec![];
        let vec_with_events = vec![event.clone(), event2.clone()];

        assert_eq!(
            empty_vec,
            storage.iter_events_queue().cloned().collect::<Vec<_>>()
        );
        storage.put_event_to_queue(event);
        storage.put_event_to_queue(event2);
        assert_eq!(
            vec_with_events,
            storage.iter_events_queue().cloned().collect::<Vec<_>>()
        );
        storage.clear_events_queue();
        assert_eq!(
            empty_vec,
            storage.iter_events_queue().cloned().collect::<Vec<_>>()
        );
    }

    #[test]
    fn blocking_and_unblocking_account_tests() {
        let TOKEN_ID_ETH = U256::from(0);
        let mut storage = ControllerStorage::new();
        let address = H160::from_slice(&ETH_ADDRESS);
        let event = Event::EthRelayMessage(
            H256::from_slice(&MESSAGE_ID),
            address,
            H256::from_slice(&SUB_ADDRESS),
            AMOUNT.into(),
            TOKEN_ID_ETH,
            BLOCK_NUMBER,
        );
        let event2 = Event::EthRelayMessage(
            H256::from_slice(&MESSAGE_ID2),
            address,
            H256::from_slice(&SUB_ADDRESS),
            AMOUNT.into(),
            TOKEN_ID_ETH,
            BLOCK_NUMBER,
        );
        let empty_vec: Vec<Event> = vec![];
        let vec_with_events = vec![event.clone(), event2.clone()];

        assert_eq!(
            empty_vec,
            storage.iter_events_queue().cloned().collect::<Vec<_>>()
        );
        assert_eq!(
            false,
            storage.is_account_blocked(Some(Address::Eth(address)))
        );
        storage.block_account(Address::Eth(address));
        assert_eq!(
            true,
            storage.is_account_blocked(Some(Address::Eth(address)))
        );

        storage.put_event_to_account_queue(event);
        storage.put_event_to_account_queue(event2);
        assert_eq!(
            empty_vec,
            storage.iter_events_queue().cloned().collect::<Vec<_>>()
        );
        storage.unblock_account(Address::Eth(address));
        assert_eq!(
            false,
            storage.is_account_blocked(Some(Address::Eth(address)))
        );
        assert_eq!(
            vec_with_events,
            storage.iter_events_queue().cloned().collect::<Vec<_>>()
        );
    }
}
