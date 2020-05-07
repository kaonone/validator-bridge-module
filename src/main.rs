use dotenv::dotenv;
use env_logger;
//use log;
use std::sync::mpsc::channel;

mod config;
mod controller;
mod controller_storage;
mod ethereum_transactions;
mod executor;
mod graph_node_event_listener;
mod oracle;
mod substrate_event_listener;
mod substrate_transactions;

pub const FETCHED_CRYPTOS: [(&[u8], &[u8], &[u8]); 4] = [
    (
        b"DAI",
        b"cryptocompare",
        b"https://min-api.cryptocompare.com/data/price?fsym=DAI&tsyms=USD",
    ),
    (
        b"USDT",
        b"cryptocompare",
        b"https://min-api.cryptocompare.com/data/price?fsym=USDT&tsyms=USD",
    ),
    (
        b"USDC",
        b"cryptocompare",
        b"https://min-api.cryptocompare.com/data/price?fsym=USDC&tsyms=USD",
    ),
    (
        b"cDAI",
        b"coingecko",
        b"https://api.coingecko.com/api/v3/simple/price?ids=cDAI&vs_currencies=USD",
    ),
];

fn main() {
    env_logger::init();
    dotenv().ok();

    let config = config::Config::load().expect("can not load config");

    let (controller_tx, controller_rx) = channel();
    let (executor_tx, executor_rx) = channel();

    let controller_thread = controller::spawn(config.clone(), controller_rx, executor_tx);
    let executor_thread = executor::spawn(config.clone(), executor_rx);
    let graph_node_event_listener_thread =
        graph_node_event_listener::spawn(config.clone(), controller_tx.clone());
    let oracle_event_listener_thread =
        oracle::spawn(config.clone(), &FETCHED_CRYPTOS, controller_tx.clone());
    let substrate_event_listener_thread = substrate_event_listener::spawn(config, controller_tx);

    let _ = controller_thread.join().expect("controller thread failed");
    let _ = executor_thread.join().expect("executor thread failed");
    let _ = graph_node_event_listener_thread
        .join()
        .expect("graph node thread failed");
    let _ = oracle_event_listener_thread
        .join()
        .expect("oracle thread failed");
    let _ = substrate_event_listener_thread
        .join()
        .expect("substrate thread failed");
}

#[cfg(test)]
mod tests {
    use super::*;
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

        println!(
            "previous:{:?} genesis:{:?}",
            genesis,
            genesis_hash.to_string()
        );
        assert_eq!(genesis, genesis_hash.to_string());
    }

    #[test]
    fn graph_listener_test() {
        dotenv().ok();
        let config = config::Config::load().expect("can not load config");
        let (controller_tx, controller_rx) = channel();
        let (executor_tx, executor_rx) = channel();

        let controller_thread = controller::spawn(config.clone(), controller_rx, executor_tx);
        let executor_thread = executor::spawn(config.clone(), executor_rx);
        let graph_node_event_listener_thread =
            graph_node_event_listener::spawn(config.clone(), controller_tx.clone());

        let _ = controller_thread.join().expect("controller thread failed");
        let _ = executor_thread.join().expect("executor thread failed");
        let _ = graph_node_event_listener_thread
            .join()
            .expect("graph node thread failed");
    }
}
