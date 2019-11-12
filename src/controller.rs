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
type BlockNumber = u128;
type Timestamp = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    EthBridgePausedMessage(MessageId, BlockNumber),
    EthBridgeResumedMessage(MessageId, BlockNumber),
    EthBridgeStartedMessage(MessageId, EthAddress, BlockNumber),
    EthBridgeStoppedMessage(MessageId, EthAddress, BlockNumber),

    EthRelayMessage(MessageId, EthAddress, SubAddress, Amount, BlockNumber),
    EthApprovedRelayMessage(MessageId, EthAddress, SubAddress, Amount, BlockNumber),
    EthRevertMessage(MessageId, EthAddress, Amount, BlockNumber),
    EthWithdrawMessage(MessageId, BlockNumber),

    EthValidatorAddedMessage(MessageId, SubAddress, BlockNumber),
    EthValidatorRemovedMessage(MessageId, SubAddress, BlockNumber),

    EthHostAccountPausedMessage(MessageId, EthAddress, Timestamp, BlockNumber),
    EthHostAccountResumedMessage(MessageId, EthAddress, Timestamp, BlockNumber),
    EthGuestAccountPausedMessage(MessageId, SubAddress, Timestamp, BlockNumber),
    EthGuestAccountResumedMessage(MessageId, SubAddress, Timestamp, BlockNumber),

    SubRelayMessage(MessageId, BlockNumber),
    SubApprovedRelayMessage(MessageId, SubAddress, EthAddress, Amount, BlockNumber),
    SubBurnedMessage(MessageId, SubAddress, EthAddress, Amount, BlockNumber),
    SubMintedMessage(MessageId, BlockNumber),
    SubCancellationConfirmedMessage(MessageId, BlockNumber),

    SubAccountPausedMessage(MessageId, SubAddress, Timestamp, BlockNumber),
    SubAccountResumedMessage(MessageId, SubAddress, Timestamp, BlockNumber),
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
            Self::EthBridgePausedMessage(message_id, _) => message_id,
            Self::EthBridgeResumedMessage(message_id, _) => message_id,
            Self::EthBridgeStartedMessage(message_id, _, _) => message_id,
            Self::EthBridgeStoppedMessage(message_id, _, _) => message_id,
            Self::EthRelayMessage(message_id, _, _, _, _) => message_id,
            Self::EthApprovedRelayMessage(message_id, _, _, _, _) => message_id,
            Self::EthRevertMessage(message_id, _, _, _) => message_id,
            Self::EthWithdrawMessage(message_id, _) => message_id,
            Self::EthValidatorAddedMessage(message_id, _, _) => message_id,
            Self::EthValidatorRemovedMessage(message_id, _, _) => message_id,
            Self::EthHostAccountPausedMessage(message_id, _, _, _) => message_id,
            Self::EthHostAccountResumedMessage(message_id, _, _, _) => message_id,
            Self::EthGuestAccountPausedMessage(message_id, _, _, _) => message_id,
            Self::EthGuestAccountResumedMessage(message_id, _, _, _) => message_id,
            Self::SubRelayMessage(message_id, _) => message_id,
            Self::SubApprovedRelayMessage(message_id, _, _, _, _) => message_id,
            Self::SubBurnedMessage(message_id, _, _, _, _) => message_id,
            Self::SubMintedMessage(message_id, _) => message_id,
            Self::SubCancellationConfirmedMessage(message_id, _) => message_id,
            Self::SubAccountPausedMessage(message_id, _, _, _) => message_id,
            Self::SubAccountResumedMessage(message_id, _, _, _) => message_id,
        }
    }

    pub fn block_number(&self) -> u128 {
        match self {
            Self::EthBridgePausedMessage(_, block_number) => *block_number,
            Self::EthBridgeResumedMessage(_, block_number) => *block_number,
            Self::EthBridgeStartedMessage(_, _, block_number) => *block_number,
            Self::EthBridgeStoppedMessage(_, _, block_number) => *block_number,
            Self::EthRelayMessage(_, _, _, _, block_number) => *block_number,
            Self::EthApprovedRelayMessage(_, _, _, _, block_number) => *block_number,
            Self::EthRevertMessage(_, _, _, block_number) => *block_number,
            Self::EthWithdrawMessage(_, block_number) => *block_number,
            Self::EthValidatorAddedMessage(_, _, block_number) => *block_number,
            Self::EthValidatorRemovedMessage(_, _, block_number) => *block_number,
            Self::EthHostAccountPausedMessage(_, _, _, block_number) => *block_number,
            Self::EthHostAccountResumedMessage(_, _, _, block_number) => *block_number,
            Self::EthGuestAccountPausedMessage(_, _, _, block_number) => *block_number,
            Self::EthGuestAccountResumedMessage(_, _, _, block_number) => *block_number,
            Self::SubRelayMessage(_, block_number) => *block_number,
            Self::SubApprovedRelayMessage(_, _, _, _, block_number) => *block_number,
            Self::SubBurnedMessage(_, _, _, _, block_number) => *block_number,
            Self::SubMintedMessage(_, block_number) => *block_number,
            Self::SubCancellationConfirmedMessage(_, block_number) => *block_number,
            Self::SubAccountPausedMessage(_, _, _, block_number) => *block_number,
            Self::SubAccountResumedMessage(_, _, _, block_number) => *block_number,
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
                            storage.iter_events_queue().cloned().for_each(|event| {
                                executor_tx.send(event).expect("can not sent event")
                            });
                            storage.clear_events_queue();
                            executor_tx.send(event).expect("can not sent event")
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
