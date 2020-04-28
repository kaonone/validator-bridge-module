use dotenv::dotenv;
use env_logger;

use std::sync::mpsc::channel;

mod config;
mod controller;
mod controller_storage;
mod ethereum_transactions;
mod executor;
mod graph_node_event_listener;
mod substrate_event_listener;
mod substrate_transactions;

fn main() {
    env_logger::init();
    dotenv().ok();

    let config = config::Config::load().expect("can not load config");

    let (controller_tx, controller_rx) = channel();
    let (executor_tx, executor_rx) = channel();

    let controller_thread = controller::spawn(config.clone(), controller_rx, executor_tx);
    log::info!("spawned controller thread");
    let executor_thread = executor::spawn(config.clone(), executor_rx);
    log::info!("spawned executor thread");
    let graph_node_event_listener_thread =
    graph_node_event_listener::spawn(config.clone(), controller_tx.clone());
    log::info!("spawned graph node listener thread");
    let substrate_event_listener_thread = substrate_event_listener::spawn(config, controller_tx);
    log::info!("spawned substrate event listener thread");

    let _ = controller_thread.join();
    let _ = executor_thread.join();
    let _ = graph_node_event_listener_thread.join();
    let _ = substrate_event_listener_thread.join();
}

#[cfg(test)]
mod tests {
    use crate::substrate_transactions::get_sr25519_pair;
    use substrate_api_client::Api;

    /// the whole purpose of the test to address some chain's runtime
    /// so we are comparing actual zero block's hash with the one we
    /// get with this api
    #[test]
    fn integration_test() {
        // genesis block for Kusama CC3
        // can be found here: https://polkadot.js.org/apps/#/explorer
        let block = "0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe";
        let genesis = format!("{}…{}", &block[0..6], &block[62..66]);
        let test_mnemonic_phrase =
            "breeze two blade day chalk grief alert crime coach donate sister escape";
        let url = String::from("wss://kusama-rpc.polkadot.io/");

        let sub_api = Api::new(url).set_signer(get_sr25519_pair(test_mnemonic_phrase));
        let genesis_hash = sub_api.get_genesis_hash();

        println!("previous:{:?} genesis:{:?}", genesis, genesis_hash.to_string());
        assert_eq!(genesis, genesis_hash.to_string());
    }
}
