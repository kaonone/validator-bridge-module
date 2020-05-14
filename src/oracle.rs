use crate::{config::Config, controller::Event};
use log;
use node_runtime::Balance;
use serde_json::{self, Value};
use std::{collections::HashMap, sync::mpsc::Sender, thread, time::Duration};
use web3::types::H256;

#[derive(Debug)]
struct Oracle {
    config: Config,
    tokens: HashMap<String, (String, String, String)>,
    controller_tx: Sender<Event>,
}

pub fn spawn(
    config: Config,
    tokens: &'static [(&[u8], &[u8], &[u8])],
    controller_tx: Sender<Event>,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("oracle".to_string())
        .spawn(move || {
            let map = tokens
                .iter()
                .map(|t| {
                    (
                        String::from_utf8(t.0.to_owned())
                            .expect("Failed to parse crypto symbol to fetch"),
                        String::from_utf8(t.1.to_owned())
                            .expect("Failed to parse crypto source to fetch"),
                        String::from_utf8(t.2.to_owned())
                            .expect("Failed to parse crypto url to fetch"),
                    )
                })
                .map(|t| (t.0.clone(), t.clone()))
                .collect::<HashMap<String, (String, String, String)>>();
            let mut oracle = Oracle::new(config, map, controller_tx);
            oracle.start();
        })
        .expect("can not start oracle")
}

impl Oracle {
    fn new(
        config: Config,
        tokens: HashMap<String, (String, String, String)>,
        controller_tx: Sender<Event>,
    ) -> Self {
        Oracle {
            config,
            tokens,
            controller_tx,
        }
    }

    fn start(&mut self) {
        log::info!("starting oracle");
        self.start_polling();
    }

    fn start_polling(&self) {
        let sym = &self.config.token_symbol;
        let token = self.tokens.get(sym).unwrap();
        let client = reqwest::Client::new();
        loop {
            let req = client.get(&token.2).send();
            let res = req
                .expect("Failed to send fetch crypto request")
                .text()
                .expect("Failed to parse fetch crypto request to text");
            let json: Value =
                serde_json::from_str(&res).expect("Failed to parse json from response");
            log::debug!(
                "Oracle response json ({}-{}): {:?}",
                &token.1,
                &token.0,
                json
            );
            let price = match token.1.clone() {
                s if s == "cryptocompare" => self.parse_price_from_cryptocompare(json),
                s if s == "coingecko" => self.parse_price_from_coingecko(json, &token.0),
                _ => todo!(),
            };

            log::info!(
                "Oracle parse result ({}-{}): {:?}",
                &token.1,
                &token.0,
                price
            );

            let hash = H256::default();
            let event = Event::OracleMessage(hash, token.0.as_bytes().to_vec(), price);
            self.controller_tx
                .send(event.clone())
                .expect("Failed to sent Oracle message");

            log::debug!("Sent Event:{:?}", event);

            thread::sleep(Duration::from_secs(6));
        }
    }
    fn round_value(v: f64) -> Balance {
        let mut precisioned: u128 = (v * 1000000000.0).round() as u128;
        precisioned = precisioned * 1000000000; // saturate to 10^18 precision
        let balance = precisioned.into();
        balance
    }

    fn parse_price_from_cryptocompare(&self, v: Value) -> Balance {
        // Expected JSON shape:
        //   r#"{"USD": 7064.16}"#;
        log::debug!("cryptocompare:{:?}", v);
        let val_f64: f64 = v["USD"].as_f64().map_or(0.0, |f| f);
        Self::round_value(val_f64)
    }

    fn parse_price_from_coingecko(&self, v: Value, token: &str) -> Balance {
        // Expected JSON shape:
        //   r#"{"cdai":{"usd": 7064.16}}"#;
        log::debug!("coingecko:{:?}", v);
        let v = &v[token.to_lowercase()];
        let val_f64: f64 = v["usd"].as_f64().map_or(0.0, |f| f);
        Self::round_value(val_f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FETCHED_CRYPTOS;
    use std::sync::mpsc::channel;
    use web3::types::Address;

    // #[ignore] // test https fetching, goes over 60s. Run explicitly with --nocapture flag
    #[test]
    fn try_fetch_cdai() {
        let (s, r) = channel::<Event>();
        let config: Config = Config {
            graph_node_api_url: String::default(),
            eth_api_url: String::default(),
            eth_validator_address: Address::default(),
            eth_validator_private_key: String::default(),
            token_bridge_address: Address::default(),
            token_symbol: String::from("cDAI"),
            eth_gas_price: u64::default(),
            eth_gas: u64::default(),
            sub_token_index: u32::default(),
            sub_api_url: String::default(),
            sub_validator_mnemonic_phrase: String::default(),
        };
        let oracle_event_listener_thread = spawn(config.clone(), &FETCHED_CRYPTOS, s);

        let _ = oracle_event_listener_thread.join();
        thread::sleep(Duration::from_secs(11));

        r.iter().for_each(|e| {
            println!("RECEIVED:{:?}", e);
        });
        assert_eq!(1, 0);
    }
    // #[ignore] // test https fetching, goes over 60s. Run explicitly with --nocapture flag
    #[test]
    fn try_fetch_dai() {
        let (s, r) = channel::<Event>();
        let config: Config = Config {
            graph_node_api_url: String::default(),
            eth_api_url: String::default(),
            eth_validator_address: Address::default(),
            eth_validator_private_key: String::default(),
            token_bridge_address: Address::default(),
            token_symbol: String::from("DAI"),
            eth_gas_price: u64::default(),
            eth_gas: u64::default(),
            sub_token_index: u32::default(),
            sub_api_url: String::default(),
            sub_validator_mnemonic_phrase: String::default(),
        };
        let oracle_event_listener_thread = spawn(config.clone(), &FETCHED_CRYPTOS, s);

        let _ = oracle_event_listener_thread.join();
        thread::sleep(Duration::from_secs(11));

        r.iter().for_each(|e| {
            println!("RECEIVED:{:?}", e);
        });
        assert_eq!(1, 0);
    }

    #[test]
    fn test_simple_parsing() {
        let json: Vec<u8> = r#"{"USD": 7064.16}"#.as_bytes().to_vec();
        let v: Value = serde_json::from_str(
            &core::str::from_utf8(&json)
                .map_err(|_| "JSON result cannot convert to string")
                .expect("fail"),
        )
        .map_err(|_| "JSON parsing error")
        .expect("double fail");
        let result = v["USD"].as_f64().unwrap();

        assert_eq!(result, 7064.16);
    }
}
