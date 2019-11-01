use log;
use web3::{
    contract::{Contract, Options},
    futures::Future,
    types::{H256, U256},
};

use std::{sync::mpsc::Sender, thread, time::Duration};

use crate::config::Config;
use crate::controller::Event;

struct EventListener {
    config: Config,
    controller_tx: Sender<Event>,
}

pub fn spawn(config: Config, controller_tx: Sender<Event>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("ethereum_event_listener".to_string())
        .spawn(move || {
            let event_listener = EventListener::new(config, controller_tx);
            event_listener.start();
        })
        .expect("can not started ethereum_event_listener")
}

impl EventListener {
    fn new(config: Config, controller_tx: Sender<Event>) -> Self {
        EventListener {
            config,
            controller_tx,
        }
    }

    fn start(&self) {
        let (_eloop, transport) =
            web3::transports::WebSocket::new(&self.config.eth_api_url).unwrap();
        let web3 = web3::Web3::new(transport);

        let contract_abi = include_bytes!("../res/EthContract.abi");
        let contract =
            Contract::from_json(web3.eth(), self.config.eth_contract_address, contract_abi)
                .expect("can not create contract");

        let fut = contract.query("bridgeStatus", (), None, Options::default(), None);
        let bridge_status: U256 = fut.wait().expect("can not read bridge status");
        log::info!("got bridge status: {:?}", bridge_status);
        self.controller_tx
            .send(build_bridge_status_event(bridge_status))
            .expect("can not send event");

        loop {
            thread::sleep(Duration::from_millis(1000));
        }
    }
}

fn build_bridge_status_event(bridge_status: U256) -> Event {
    const MESSAGE_ID: [u8; 32] = [0; 32];
    const ETH_BLOCK_NUMBER: u128 = 0;
    match bridge_status.low_u64() {
        0 => Event::EthBridgeStartedMessage(parse_h256(&MESSAGE_ID), ETH_BLOCK_NUMBER),
        1 => Event::EthBridgePausedMessage(parse_h256(&MESSAGE_ID), ETH_BLOCK_NUMBER),
        _ => Event::EthBridgeStoppedMessage(parse_h256(&MESSAGE_ID), ETH_BLOCK_NUMBER),
    }
}

fn parse_h256(hash: &[u8]) -> H256 {
    H256::from_slice(hash)
}
