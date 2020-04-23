use log;
use web3::types::{H160, H256, U256};

use codec::Decode;
use node_runtime::{bridge, bridge::RawEvent as BridgeEvent, AccountId};
use primitives::{self, sr25519};
use substrate_api_client::{
    events::{EventsDecoder, RuntimeEvent},
    utils::hexstr_to_vec,
    Api,
};

use std::convert::TryFrom;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::config::Config;
use crate::controller::Event;

#[derive(Debug, Clone)]
struct EventListener {
    config: Config,
    events_in: Sender<String>,
}

struct EventHandler {
    config: Config,
    controller_tx: Sender<Event>,
    events_out: Receiver<String>,
}

pub fn spawn(config: Config, controller_tx: Sender<Event>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("substrate_event_processor".to_string())
        .spawn(move || {
            let (events_in, events_out) = channel();
            let config2 = config.clone();
            let event_listener = thread::Builder::new()
                .name("substrate_event_listener".to_string())
                .spawn(move || {
                    let event_listener = EventListener::new(config, events_in);
                    event_listener.start();
                })
                .expect("can not start substrate_event_listener");

            let event_handler = thread::Builder::new()
                .name("substrate_event_handler".to_string())
                .spawn(move || {
                    let event_handler = EventHandler::new(config2, controller_tx, events_out);
                    event_handler.start();
                })
                .expect("can not start substrate_event_handler");

            let _ = event_listener.join();
            let _ = event_handler.join();
        })
        .expect("can not start substrate_event_processor")
}

impl EventListener {
    fn new(config: Config, events_in: Sender<String>) -> Self {
        EventListener { config, events_in }
    }

    fn start(&self) {
        let sub_api = Api::<sr25519::Pair>::new(self.config.sub_api_url.clone());
        sub_api.subscribe_events(self.events_in.clone());
    }
}

impl EventHandler {
    fn new(config: Config, controller_tx: Sender<Event>, events_out: Receiver<String>) -> Self {
        EventHandler {
            config,
            controller_tx,
            events_out,
        }
    }

    fn start(&self) {
        self.events_out.iter().for_each(|event| {
            log::debug!("[substrate] got event: {:?}", event);

            let unhex = hexstr_to_vec(event).expect("convert hexstr to vec failed");
            let mut er_enc = unhex.as_slice();

            let sub_api = Api::<sr25519::Pair>::new(self.config.sub_api_url.clone());
            let event_decoder = EventsDecoder::try_from(sub_api.metadata).unwrap();
            let events = event_decoder.decode_events(&mut er_enc);

            match events {
                Ok(raw_events) => {
                    for (phase, event) in &raw_events {
                        log::debug!("[substrate] decoded: phase {:?} event {:?}", phase, event);
                        match event {
                            RuntimeEvent::Raw(raw) => {
                                if raw.module == "Bridge" {
                                    self.handle_bridge_event(
                                        Decode::decode(&mut &raw.data[..]).expect("decoded event"),
                                    )
                                } else {
                                    log::debug!(
                                        "[substrate] ignoring unsupported module event: {:?}",
                                        event
                                    )
                                }
                            }
                            _ => log::debug!("ignoring unsupported module event: {:?}", event),
                        }
                    }
                }
                Err(_) => log::error!("[substrate] could not decode event record list"),
            }
        })
    }

    fn handle_bridge_event(&self, event: BridgeEvent<AccountId, primitives::H256, u128, u32>) {
        const BLOCK_NUMBER: u128 = 0;

        log::info!("[substrate] bridge event: {:?}", event);
        match &event {
            bridge::RawEvent::RelayMessage(message_id) => {
                let event =
                    Event::SubRelayMessage(H256::from_slice(message_id.as_bytes()), BLOCK_NUMBER);
                self.controller_tx.send(event).expect("can not send event");
            }
            bridge::RawEvent::ApprovedRelayMessage(message_id, token_id, from, to, amount) => {
                let from: [u8; 32] = from.to_owned().into();
                let event = Event::SubApprovedRelayMessage(
                    H256::from_slice(message_id.as_bytes()),
                    H256::from(from),
                    H160::from_slice(to.as_bytes()),
                    U256::from(*token_id),
                    U256::from(*amount),
                    BLOCK_NUMBER,
                );
                self.controller_tx.send(event).expect("can not send event");
            }
            bridge::RawEvent::BurnedMessage(message_id, token_id, from, to, amount) => {
                let from: [u8; 32] = from.to_owned().into();
                let event = Event::SubBurnedMessage(
                    H256::from_slice(message_id.as_bytes()),
                    H256::from(from),
                    H160::from_slice(to.as_bytes()),
                    U256::from(*amount),
                    U256::from(*token_id),
                    BLOCK_NUMBER,
                );
                self.controller_tx.send(event).expect("can not send event");
            }
            bridge::RawEvent::MintedMessage(message_id, token_id) => {
                let event = Event::SubMintedMessage(
                    H256::from_slice(message_id.as_bytes()),
                    U256::from(*token_id),
                    BLOCK_NUMBER,
                );
                self.controller_tx.send(event).expect("can not send event");
            }
            bridge::RawEvent::CancellationConfirmedMessage(message_id, token_id) => {
                let event = Event::SubCancellationConfirmedMessage(
                    H256::from_slice(message_id.as_bytes()),
                    U256::from(*token_id),
                    BLOCK_NUMBER,
                );
                self.controller_tx.send(event).expect("can not send event");
            }
            bridge::RawEvent::AccountPausedMessage(
                message_id,
                sub_address,
                timestamp,
                token_id,
            ) => {
                let sub_address: [u8; 32] = sub_address.to_owned().into();
                let event = Event::SubAccountPausedMessage(
                    H256::from_slice(message_id.as_bytes()),
                    H256::from(sub_address),
                    u64::from(*timestamp),
                    U256::from(*token_id),
                    BLOCK_NUMBER,
                );
                self.controller_tx.send(event).expect("can not send event");
            }
            bridge::RawEvent::AccountResumedMessage(
                message_id,
                sub_address,
                timestamp,
                token_id,
            ) => {
                let sub_address: [u8; 32] = sub_address.to_owned().into();
                let event = Event::SubAccountResumedMessage(
                    H256::from_slice(message_id.as_bytes()),
                    H256::from(sub_address),
                    u64::from(*timestamp),
                    U256::from(*token_id),
                    BLOCK_NUMBER,
                );
                self.controller_tx.send(event).expect("can not send event");
            }
        }
    }
}
