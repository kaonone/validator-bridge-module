use web3::types::{H160, H256, U256};

use log;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::config::Config;
use crate::controller_storage::ControllerStorage;

type MessageId = H256;
type EthAddress = H160;
type SubAddress = H256;
type Amount = U256;
type TokenId = U256;
type BlockNumber = u128;
type Timestamp = u64;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Address {
    Eth(EthAddress),
    Sub(SubAddress),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    EthBridgePausedMessage(MessageId, BlockNumber),
    EthBridgeResumedMessage(MessageId, BlockNumber),
    EthBridgeStartedMessage(MessageId, EthAddress, BlockNumber),
    EthBridgeStoppedMessage(MessageId, EthAddress, BlockNumber),

    EthRelayMessage(
        MessageId,
        EthAddress,
        SubAddress,
        Amount,
        TokenId,
        BlockNumber,
    ),
    EthApprovedRelayMessage(
        MessageId,
        EthAddress,
        SubAddress,
        Amount,
        TokenId,
        BlockNumber,
    ),
    EthRevertMessage(MessageId, EthAddress, Amount, BlockNumber),
    EthWithdrawMessage(MessageId, BlockNumber),

    EthHostAccountPausedMessage(MessageId, EthAddress, Timestamp, BlockNumber),
    EthHostAccountResumedMessage(MessageId, EthAddress, Timestamp, BlockNumber),
    EthGuestAccountPausedMessage(MessageId, SubAddress, Timestamp, BlockNumber),
    EthGuestAccountResumedMessage(MessageId, SubAddress, Timestamp, BlockNumber),

    EthSetNewLimits(
        MessageId,
        Amount,
        Amount,
        Amount,
        Amount,
        Amount,
        Amount,
        Amount,
        Amount,
        Amount,
        Amount,
        BlockNumber,
    ),
    
    EthValidatorsListMessage(MessageId, Vec<SubAddress>, Amount, BlockNumber),

    SubRelayMessage(MessageId, BlockNumber),
    SubApprovedRelayMessage(
        MessageId,
        SubAddress,
        EthAddress,
        Amount,
        TokenId,
        BlockNumber,
    ),
    SubBurnedMessage(
        MessageId,
        SubAddress,
        EthAddress,
        Amount,
        TokenId,
        BlockNumber,
    ),
    SubMintedMessage(MessageId, TokenId, BlockNumber),
    SubCancellationConfirmedMessage(MessageId, TokenId, BlockNumber),

    SubAccountPausedMessage(MessageId, SubAddress, Timestamp, TokenId, BlockNumber),
    SubAccountResumedMessage(MessageId, SubAddress, Timestamp, TokenId, BlockNumber),
}

#[derive(Debug, PartialEq, Eq)]
enum EventType {
    Transfer,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    NotReady,
    Active,
    Paused,
    Stopped,
}

#[derive(Debug)]
struct Controller {
    config: Config,
    status: Status,
    controller_rx: Receiver<Event>,
    executor_tx: Sender<Event>,
    storage: ControllerStorage,
}

pub fn spawn(
    config: Config,
    controller_rx: Receiver<Event>,
    executor_tx: Sender<Event>,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("controller".to_string())
        .spawn(move || {
            let mut controller = Controller::new(config, controller_rx, executor_tx);
            controller.start();
        })
        .expect("can not started controller")
}

impl Event {
    pub fn message_id(&self) -> &H256 {
        match self {
            // Transfers
            Self::EthRelayMessage(message_id, _, _, _, _, _) => message_id,
            Self::EthApprovedRelayMessage(message_id, _, _, _, _, _) => message_id,
            Self::EthRevertMessage(message_id, _, _, _) => message_id,
            Self::EthWithdrawMessage(message_id, _) => message_id,
            Self::SubRelayMessage(message_id, _) => message_id,
            Self::SubApprovedRelayMessage(message_id, _, _, _, _, _) => message_id,
            Self::SubBurnedMessage(message_id, _, _, _, _, _) => message_id,
            Self::SubMintedMessage(message_id, _, _) => message_id,
            Self::SubCancellationConfirmedMessage(message_id, _, _) => message_id,
            // Bridge management
            Self::EthBridgePausedMessage(message_id, _) => message_id,
            Self::EthBridgeResumedMessage(message_id, _) => message_id,
            Self::EthBridgeStartedMessage(message_id, _, _) => message_id,
            Self::EthBridgeStoppedMessage(message_id, _, _) => message_id,
            Self::EthSetNewLimits(message_id, _, _, _, _, _, _, _, _, _, _, _) => message_id,
            Self::EthValidatorsListMessage(message_id, _, _, _) => message_id,
            // Account management
            Self::EthHostAccountPausedMessage(message_id, _, _, _) => message_id,
            Self::EthHostAccountResumedMessage(message_id, _, _, _) => message_id,
            Self::EthGuestAccountPausedMessage(message_id, _, _, _) => message_id,
            Self::EthGuestAccountResumedMessage(message_id, _, _, _) => message_id,
            Self::SubAccountPausedMessage(message_id, _, _, _, _) => message_id,
            Self::SubAccountResumedMessage(message_id, _, _, _, _) => message_id,
        }
    }

    pub fn block_number(&self) -> u128 {
        match self {
            // Transfers
            Self::EthRelayMessage(_, _, _, _, _, block_number) => *block_number,
            Self::EthApprovedRelayMessage(_, _, _, _, _, block_number) => *block_number,
            Self::EthWithdrawMessage(_, block_number) => *block_number,
            Self::EthRevertMessage(_, _, _, block_number) => *block_number,
            Self::SubRelayMessage(_, block_number) => *block_number,
            Self::SubApprovedRelayMessage(_, _, _, _, _, block_number) => *block_number,
            Self::SubBurnedMessage(_, _, _, _, _, block_number) => *block_number,
            Self::SubMintedMessage(_, _, block_number) => *block_number,
            Self::SubCancellationConfirmedMessage(_, _, block_number) => *block_number,
            // Bridge management
            Self::EthSetNewLimits(_, _, _, _, _, _, _, _, _, _, _, block_number) => {
                *block_number
            }
            Self::EthBridgePausedMessage(_, block_number) => *block_number,
            Self::EthBridgeResumedMessage(_, block_number) => *block_number,
            Self::EthBridgeStartedMessage(_, _, block_number) => *block_number,
            Self::EthBridgeStoppedMessage(_, _, block_number) => *block_number,
            Self::EthValidatorsListMessage(_, _, _, block_number) => *block_number,
            // Account management
            Self::EthHostAccountPausedMessage(_, _, _, block_number) => *block_number,
            Self::EthHostAccountResumedMessage(_, _, _, block_number) => *block_number,
            Self::EthGuestAccountPausedMessage(_, _, _, block_number) => *block_number,
            Self::EthGuestAccountResumedMessage(_, _, _, block_number) => *block_number,
            Self::SubAccountPausedMessage(_, _, _, _, block_number) => *block_number,
            Self::SubAccountResumedMessage(_, _, _, _, block_number) => *block_number,
        }
    }

    fn event_type(&self) -> EventType {
        match self {
            Self::EthRelayMessage(..) => EventType::Transfer,
            Self::EthApprovedRelayMessage(..) => EventType::Transfer,
            Self::EthRevertMessage(..) => EventType::Other,
            Self::EthWithdrawMessage(..) => EventType::Transfer,
            Self::SubRelayMessage(..) => EventType::Transfer,
            Self::SubApprovedRelayMessage(..) => EventType::Transfer,
            Self::SubBurnedMessage(..) => EventType::Transfer,
            Self::SubMintedMessage(..) => EventType::Transfer,
            Self::SubCancellationConfirmedMessage(..) => EventType::Other,
            _ => EventType::Other,
        }
    }

    pub fn sender(&self) -> Option<Address> {
        match self {
            Self::EthRelayMessage(_, eth_address, _, _, _, _) => Some(Address::Eth(*eth_address)),
            Self::EthApprovedRelayMessage(_, eth_address, _, _, _, _) => {
                Some(Address::Eth(*eth_address))
            }
            Self::EthRevertMessage(_, eth_address, _, _) => Some(Address::Eth(*eth_address)),
            Self::EthWithdrawMessage(_, _) => None,
            Self::SubRelayMessage(_, _) => None,
            Self::SubApprovedRelayMessage(_, sub_address, _, _, _, _) => {
                Some(Address::Sub(*sub_address))
            }
            Self::SubBurnedMessage(_, sub_address, _, _, _, _) => Some(Address::Sub(*sub_address)),
            Self::SubMintedMessage(_, _, _) => None,
            Self::SubCancellationConfirmedMessage(_, _, _) => None,
            _ => None,
        }
    }

    pub fn token_id(&self) -> U256 {
        match self {
            Self::EthApprovedRelayMessage(_, _, _, token_id, _, _) => *token_id,
            Self::SubApprovedRelayMessage(_, _, _, token_id, _, _) => *token_id,
            _ => U256::from(0),
        }
    }
}

impl Controller {
    fn new(config: Config, controller_rx: Receiver<Event>, executor_tx: Sender<Event>) -> Self {
        Controller {
            config,
            status: Status::NotReady,
            controller_rx,
            executor_tx,
            storage: ControllerStorage::new(),
        }
    }

    fn start(&mut self) {
        log::info!("current status: {:?}", self.status);
        let storage = &mut self.storage;
        let controller_rx = &self.controller_rx;
        let status = &mut self.status;
        let executor_tx = &self.executor_tx;
        controller_rx
            .iter()
            .for_each(|event| match storage.put_event(&event) {
                Ok(()) => {
                    log::info!("received event: {:?}", event);
                    change_status(status, &event);
                    match status {
                        Status::Active => {
                            handle_account_control_events(storage, &event);
                            let deferred_events =
                                storage.iter_events_queue().cloned().collect::<Vec<_>>();
                            deferred_events.iter().cloned().for_each(|event| {
                                handle_account_control_events(storage, &event);
                                executor_tx.send(event).expect("can not sent event")
                            });
                            storage.clear_events_queue();
                            if event.event_type() == EventType::Transfer
                                && storage.is_account_blocked(event.sender())
                            {
                                storage.put_event_to_account_queue(event)
                            } else {
                                executor_tx.send(event).expect("can not sent event")
                            }
                        }
                        Status::NotReady | Status::Paused | Status::Stopped => {
                            storage.put_event_to_queue(event)
                        }
                    }
                }
                Err(e) => log::debug!("controller storage error: {:?}", e),
            })
    }
}

fn change_status(status: &mut Status, event: &Event) {
    let mut status_changed = false;
    match status {
        Status::Active => match event {
            Event::EthBridgePausedMessage(..) => {
                *status = Status::Paused;
                status_changed = true;
            }
            Event::EthBridgeStoppedMessage(..) => {
                *status = Status::Stopped;
                status_changed = true;
            }
            _ => (),
        },
        Status::NotReady | Status::Paused => match event {
            Event::EthBridgeResumedMessage(..) | Event::EthBridgeStartedMessage(..) => {
                *status = Status::Active;
                status_changed = true;
            }
            _ => (),
        },
        Status::Stopped => {
            if let Event::EthBridgeStartedMessage(..) = event {
                *status = Status::Active;
                status_changed = true;
            }
        }
    }
    if status_changed {
        log::info!("current status: {:?}", status);
    }
}

fn handle_account_control_events(storage: &mut ControllerStorage, event: &Event) {
    match event {
        Event::EthHostAccountPausedMessage(_, eth_address, _, _) => {
            storage.block_account(Address::Eth(*eth_address));
            log::info!("ethereum account {:?} is blocked", eth_address);
        }
        Event::EthHostAccountResumedMessage(_, eth_address, _, _) => {
            storage.unblock_account(Address::Eth(*eth_address));
            log::info!("ethereum account {:?} is unblocked", eth_address);
        }
        Event::EthGuestAccountPausedMessage(_, sub_address, _, _) => {
            storage.block_account(Address::Sub(*sub_address));
            log::info!("substrate account {:?} is blocked", sub_address);
        }
        Event::EthGuestAccountResumedMessage(_, sub_address, _, _) => {
            storage.unblock_account(Address::Sub(*sub_address));
            log::info!("substrate account {:?} is unblocked", sub_address);
        }
        _ => (),
    }
}
