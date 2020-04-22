use futures::future::{lazy, poll_fn};
use log;
use primitives::{self, crypto::Public};
use tokio::runtime::{Runtime, TaskExecutor};
use tokio_threadpool::blocking;
use web3::{
    futures::Future,
    types::{Address, Bytes, H160, H256, U256},
};

use std::{
    collections::HashMap,
    sync::{mpsc::Receiver, Arc},
    thread,
};

use crate::config::Config;
use crate::controller::Event;
use crate::ethereum_transactions;
use crate::substrate_transactions;

const AMOUNT: u64 = 0;

#[derive(Debug, Clone)]
struct Token {
    symbol: String,
    address: H160,
    abi: Arc<ethabi::Contract>,
}

#[derive(Debug)]
struct Executor {
    config: Config,
    executor_rx: Receiver<Event>,
}

pub fn spawn(config: Config, executor_rx: Receiver<Event>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("executor".to_string())
        .spawn(move || {
            let executor = Executor::new(config, executor_rx);
            executor.start()
        })
        .expect("can not started executor")
}

impl Executor {
    fn new(config: Config, executor_rx: Receiver<Event>) -> Self {
        Executor {
            config,
            executor_rx,
        }
    }

    fn start(&self) {
        let runtime = Runtime::new().expect("can not create tokio runtime");

        let (_eloop, transport) =
            web3::transports::WebSocket::new(&self.config.eth_api_url).unwrap();
        let web3 = web3::Web3::new(transport);

        let contracts = init_abi(vec![b"DAI", b"cDAI", b"USDT", b"USDC"], &self.config);

        let web3 = Arc::new(web3);

        self.executor_rx.iter().for_each(|event| {
            log::info!("received event: {:?}", event);
            match event {
                Event::EthBridgePausedMessage(message_id, _block_number) => {
                    handle_eth_bridge_paused_message(&self.config, runtime.executor(), message_id)
                }
                Event::EthBridgeResumedMessage(message_id, _block_number) => {
                    handle_eth_bridge_resumed_message(&self.config, runtime.executor(), message_id)
                }
                Event::EthBridgeStartedMessage(message_id, _eth_address, _block_number) => {
                    handle_eth_bridge_resumed_message(&self.config, runtime.executor(), message_id)
                }
                Event::EthBridgeStoppedMessage(message_id, _eth_address, _block_number) => {
                    handle_eth_bridge_paused_message(&self.config, runtime.executor(), message_id)
                }
                Event::EthRelayMessage(
                    message_id,
                    eth_address,
                    sub_address,
                    token_id,
                    amount,
                    _block_number,
                ) => {
                    let abi = get_contract_abi(
                        contracts.clone(),
                        get_token_id_by_address(token_id, &self.config),
                    );
                    handle_eth_relay_message(
                        &self.config,
                        runtime.executor(),
                        web3.clone(),
                        abi,
                        message_id,
                        eth_address,
                        sub_address,
                        amount,
                        token_id,
                    )
                }
                Event::EthApprovedRelayMessage(
                    message_id,
                    eth_address,
                    sub_address,
                    token_id,
                    amount,
                    _block_number,
                ) => handle_eth_approved_relay_message(
                    &self.config,
                    runtime.executor(),
                    message_id,
                    eth_address,
                    sub_address,
                    get_token_id_by_address(token_id, &self.config),
                    amount,
                ),
                Event::EthRevertMessage(message_id, _eth_address, _amount, _block_number) => {
                    handle_eth_revert_message(&self.config, runtime.executor(), message_id)
                }
                Event::EthWithdrawMessage(message_id, _block_number) => {
                    handle_eth_withdraw_message(&self.config, runtime.executor(), message_id)
                }
                Event::EthHostAccountPausedMessage(_, _, _, _) => (),
                Event::EthHostAccountResumedMessage(_, _, _, _) => (),
                Event::EthGuestAccountPausedMessage(_, _, _, _) => (),
                Event::EthGuestAccountResumedMessage(_, _, _, _) => (),
                Event::EthSetNewLimits(
                    message_id,
                    _min_host_transaction_value,
                    _max_host_transaction_value,
                    _day_host_max_limit,
                    _day_host_max_limit_for_one_address,
                    _max_host_pending_transaction_limit,
                    min_guest_transaction_value,
                    max_guest_transaction_value,
                    day_guest_max_limit,
                    day_guest_max_limit_for_one_address,
                    max_guest_pending_transaction_limit,
                    _block_number,
                ) => handle_eth_set_new_limits(
                    &self.config,
                    runtime.executor(),
                    message_id,
                    min_guest_transaction_value,
                    max_guest_transaction_value,
                    day_guest_max_limit,
                    day_guest_max_limit_for_one_address,
                    max_guest_pending_transaction_limit,
                ),
                Event::EthValidatorsListMessage(
                    message_id,
                    new_validators,
                    new_how_many_validators_decide,
                    _block_number,
                ) => handle_eth_validators_list_message(
                    &self.config,
                    runtime.executor(),
                    message_id,
                    new_validators,
                    new_how_many_validators_decide,
                ),
                Event::SubRelayMessage(message_id, _block_number) => {
                    handle_sub_relay_message(&self.config, runtime.executor(), message_id)
                }
                Event::SubApprovedRelayMessage(
                    message_id,
                    sub_address,
                    eth_address,
                    token_id,
                    amount,
                    _block_number,
                ) => handle_sub_approved_relay_message(
                    &self.config,
                    runtime.executor(),
                    web3.clone(),
                    get_contract_abi(contracts.clone(), token_id.low_u32()),
                    message_id,
                    sub_address,
                    eth_address,
                    amount,
                    token_id,
                ),
                Event::SubBurnedMessage(
                    message_id,
                    _sub_address,
                    _eth_address,
                    _amount,
                    token_id,
                    _block_number,
                ) => handle_sub_burned_message(
                    &self.config,
                    runtime.executor(),
                    web3.clone(),
                    get_contract_abi(contracts.clone(), token_id.low_u32()),
                    message_id,
                    token_id,
                ),
                Event::SubMintedMessage(message_id, token_id, _block_number) => {
                    handle_sub_minted_message(
                        &self.config,
                        runtime.executor(),
                        web3.clone(),
                        get_contract_abi(contracts.clone(), token_id.low_u32()),
                        message_id,
                        token_id,
                    )
                }
                Event::SubCancellationConfirmedMessage(message_id, token_id, _block_number) => {
                    handle_sub_cancellation_confirmed_message(
                        &self.config,
                        runtime.executor(),
                        web3.clone(),
                        get_contract_abi(contracts.clone(), token_id.low_u32()),
                        message_id,
                        token_id,
                    )
                }
                Event::SubAccountPausedMessage(
                    message_id,
                    sub_address,
                    _timestamp,
                    token_id,
                    _block_number,
                ) => handle_sub_account_paused_message(
                    &self.config,
                    runtime.executor(),
                    web3.clone(),
                    get_contract_abi(contracts.clone(), token_id.low_u32()),
                    message_id,
                    sub_address,
                    token_id,
                ),
                Event::SubAccountResumedMessage(
                    message_id,
                    sub_address,
                    _timestamp,
                    token_id,
                    _block_number,
                ) => handle_sub_account_resumed_message(
                    &self.config,
                    runtime.executor(),
                    web3.clone(),
                    get_contract_abi(contracts.clone(), token_id.low_u32()),
                    message_id,
                    sub_address,
                    token_id,
                ),
            }
        })
    }
}

fn handle_eth_bridge_paused_message(
    config: &Config,
    task_executor: TaskExecutor,
    message_id: H256,
) {
    let message_id = primitives::H256::from_slice(&message_id.to_fixed_bytes());
    let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();
    let sub_api_url = config.sub_api_url.clone();

    task_executor.spawn(lazy(move || {
        poll_fn(move || {
            blocking(|| {
                substrate_transactions::pause_bridge(
                    sub_api_url.clone(),
                    sub_validator_mnemonic_phrase.clone(),
                );
                log::info!(
                    "[substrate] called pause_bridge(), message_id: {:?}",
                    message_id
                );
            })
            .map_err(|_| panic!("the threadpool shut down"))
        })
    }));
}

fn handle_eth_bridge_resumed_message(
    config: &Config,
    task_executor: TaskExecutor,
    message_id: H256,
) {
    let message_id = primitives::H256::from_slice(&message_id.to_fixed_bytes());
    let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();
    let sub_api_url = config.sub_api_url.clone();

    task_executor.spawn(lazy(move || {
        poll_fn(move || {
            blocking(|| {
                substrate_transactions::resume_bridge(
                    sub_api_url.clone(),
                    sub_validator_mnemonic_phrase.clone(),
                );
                log::info!(
                    "[substrate] called resume_bridge(), message_id: {:?}",
                    message_id
                );
            })
            .map_err(|_| panic!("the threadpool shut down"))
        })
    }));
}

fn handle_eth_relay_message<T>(
    config: &Config,
    task_executor: TaskExecutor,
    web3: Arc<web3::Web3<T>>,
    abi: Arc<ethabi::Contract>,
    message_id: H256,
    eth_address: H160,
    sub_address: H256,
    amount: U256,
    token_id: H160,
) where
    T: web3::Transport + Send + Sync + 'static,
    T::Out: Send,
{
    let args = (message_id, eth_address, sub_address, amount);
    let eth_validator_private_key = config.eth_validator_private_key.clone();
    let token_id = get_token_id_by_address(token_id, config);
    let bridge_address = get_bridge_address(token_id, config);
    let eth_gas_price = config.eth_gas_price;
    let eth_gas = config.eth_gas;
    let data = ethereum_transactions::build_transaction_data(&abi, "approveTransfer", args);
    let fut = web3.eth().transaction_count(config.eth_validator_address, None)
        .and_then(move |nonce| {
            let tx = ethereum_transactions::build(eth_validator_private_key, bridge_address, nonce, AMOUNT, eth_gas_price, eth_gas, data);
            log::debug!("raw approveTransfer: {:?}", tx);
            web3.eth().send_raw_transaction(Bytes::from(tx))
                .then(move |res| {
                    match res {
                        Ok(tx_res) => {
                            log::info!("[ethereum] called approveTransfer({:?}, {:?}, {:?}, {:?}), nonce: {:?}, result: {:?}",
                                        message_id, eth_address, sub_address, amount, nonce, tx_res);
                        },
                        Err(err) => {
                            log::warn!("[ethereum] can not send approveTransfer({:?}, {:?}, {:?}, {:?}), nonce: {:?}, reason: {:?}",
                                        message_id, eth_address, sub_address, amount, nonce, err);
                        }
                    }
                    Ok(())
                })

        })
        .map_err(|e| log::warn!("can not get nonce: {:?}", e));
    task_executor.spawn(fut);
}

fn handle_eth_approved_relay_message(
    config: &Config,
    task_executor: TaskExecutor,
    message_id: H256,
    eth_address: H160,
    sub_address: H256,
    token_id: u32,
    amount: U256,
) {
    let message_id = primitives::H256::from_slice(&message_id.to_fixed_bytes());
    let eth_address = primitives::H160::from_slice(&eth_address.to_fixed_bytes());
    let sub_address = primitives::crypto::AccountId32::from(sub_address.to_fixed_bytes());
    let amount = amount.low_u128();
    let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();
    let sub_api_url = config.sub_api_url.clone();

    task_executor.spawn(lazy(move || {
        poll_fn(move || {
            blocking(|| {
                substrate_transactions::mint(
                    sub_api_url.clone(),
                    sub_validator_mnemonic_phrase.clone(),
                    message_id,
                    eth_address,
                    sub_address.clone(),
                    token_id,
                    amount,
                );
                log::info!(
                    "[substrate] called multi_signed_mint({:?}, {:?}, {:?}, {:?})",
                    message_id,
                    eth_address,
                    sub_address,
                    amount
                );
            })
            .map_err(|_| panic!("the threadpool shut down"))
        })
    }));
}

fn handle_eth_revert_message(config: &Config, task_executor: TaskExecutor, message_id: H256) {
    let message_id = primitives::H256::from_slice(&message_id.to_fixed_bytes());
    let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();
    let sub_api_url = config.sub_api_url.clone();

    task_executor.spawn(lazy(move || {
        poll_fn(move || {
            blocking(|| {
                substrate_transactions::cancel_transfer(
                    sub_api_url.clone(),
                    sub_validator_mnemonic_phrase.clone(),
                    message_id,
                );
                log::info!("[substrate] called cancel_transfer({:?})", message_id);
            })
            .map_err(|_| panic!("the threadpool shut down"))
        })
    }));
}

fn handle_eth_withdraw_message(config: &Config, task_executor: TaskExecutor, message_id: H256) {
    let message_id = primitives::H256::from_slice(&message_id.to_fixed_bytes());
    let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();
    let sub_api_url = config.sub_api_url.clone();

    task_executor.spawn(lazy(move || {
        poll_fn(move || {
            blocking(|| {
                substrate_transactions::confirm_transfer(
                    sub_api_url.clone(),
                    sub_validator_mnemonic_phrase.clone(),
                    message_id,
                );
                log::info!("[substrate] called confirm_transfer({:?})", message_id);
            })
            .map_err(|_| panic!("the threadpool shut down"))
        })
    }));
}

fn handle_eth_set_new_limits(
    config: &Config,
    task_executor: TaskExecutor,
    message_id: H256,
    min_guest_transaction_value: U256,
    max_guest_transaction_value: U256,
    day_guest_max_limit: U256,
    day_guest_max_limit_for_one_address: U256,
    max_guest_pending_transaction_limit: U256,
) {
    let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();
    let sub_api_url = config.sub_api_url.clone();

    task_executor.spawn(lazy(move || {
        poll_fn(move || {
            blocking(|| {
                substrate_transactions::update_limits(
                    sub_api_url.clone(),
                    sub_validator_mnemonic_phrase.clone(),
                    min_guest_transaction_value.as_u128(),
                    max_guest_transaction_value.as_u128(),
                    day_guest_max_limit.as_u128(),
                    day_guest_max_limit_for_one_address.as_u128(),
                    max_guest_pending_transaction_limit.as_u128(),
                );
                log::info!(
                    "[substrate] called update_limits({:?}, {:?}, {:?}, {:?}, {:?}), message_id: {:?}",
                    min_guest_transaction_value,
                    max_guest_transaction_value,
                    day_guest_max_limit,
                    day_guest_max_limit_for_one_address,
                    max_guest_pending_transaction_limit,
                    message_id
                );
            })
            .map_err(|_| panic!("the threadpool shut down"))
        })
    }));
}

fn handle_eth_validators_list_message(
    config: &Config,
    task_executor: TaskExecutor,
    message_id: H256,
    new_validators: Vec<H256>,
    new_how_many_validators_decide: U256,
) {
    let message_id = primitives::H256::from_slice(&message_id.to_fixed_bytes());
    let new_validators = new_validators
        .iter()
        .map(|a| primitives::sr25519::Public::from_slice(&a.to_fixed_bytes()))
        .collect::<Vec<_>>();
    let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();
    let sub_api_url = config.sub_api_url.clone();

    task_executor.spawn(lazy(move || {
        poll_fn(move || {
            blocking(|| {
                substrate_transactions::update_validator_list(
                    sub_api_url.clone(),
                    sub_validator_mnemonic_phrase.clone(),
                    message_id,
                    new_how_many_validators_decide.as_u64(),
                    new_validators.clone(),
                );
                log::info!(
                    "[substrate] called update_validator_list({:?}, {:?}, {:?})",
                    message_id,
                    new_how_many_validators_decide,
                    new_validators,
                );
            })
            .map_err(|_| panic!("the threadpool shut down"))
        })
    }));
}

fn handle_sub_relay_message(config: &Config, task_executor: TaskExecutor, message_id: H256) {
    let message_id = primitives::H256::from_slice(&message_id.to_fixed_bytes());
    let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();
    let sub_api_url = config.sub_api_url.clone();

    task_executor.spawn(lazy(move || {
        poll_fn(move || {
            blocking(|| {
                substrate_transactions::approve_transfer(
                    sub_api_url.clone(),
                    sub_validator_mnemonic_phrase.clone(),
                    message_id,
                );
                log::info!("[substrate] called approve_transfer({:?})", message_id);
            })
            .map_err(|_| panic!("the threadpool shut down"))
        })
    }));
}

fn handle_sub_approved_relay_message<T>(
    config: &Config,
    task_executor: TaskExecutor,
    web3: Arc<web3::Web3<T>>,
    abi: Arc<ethabi::Contract>,
    message_id: H256,
    sub_address: H256,
    eth_address: H160,
    amount: U256,
    token_id: U256,
) where
    T: web3::Transport + Send + Sync + 'static,
    T::Out: Send,
{
    let args = (message_id, sub_address, eth_address, amount);
    let eth_validator_private_key = config.eth_validator_private_key.clone();
    let contract_address = get_bridge_address(token_id.low_u32(), config);
    let eth_gas_price = config.eth_gas_price;
    let eth_gas = config.eth_gas;
    let data = ethereum_transactions::build_transaction_data(&abi, "withdrawTransfer", args);
    let fut = web3.eth().transaction_count(config.eth_validator_address, None)
        .and_then(move |nonce| {
            let tx = ethereum_transactions::build(eth_validator_private_key, contract_address, nonce, AMOUNT, eth_gas_price, eth_gas, data);
            log::debug!("raw withdrawTransfer: {:?}", tx);
            web3.eth().send_raw_transaction(Bytes::from(tx))
                .then(move |res| {
                    match res {
                        Ok(tx_res) => {
                            log::info!("[ethereum] called withdrawTransfer({:?}, {:?}, {:?}, {:?}), nonce: {:?}, result: {:?}",
                                       args.0, args.1, args.2, args.3, nonce, tx_res)
                        },
                        Err(err) => {
                            log::warn!("can not send withdrawTransfer({:?}, {:?}, {:?}, {:?}), nonce: {:?}, reason: {:?}",
                                       args.0, args.1, args.2, args.3, nonce, err);

                        }
                    }

                    Ok(())
                })
        })
        .or_else(|e| {
            log::warn!("can not get nonce: {:?}", e);
            Ok(())
        });
    task_executor.spawn(fut);
}

fn handle_sub_minted_message<T>(
    config: &Config,
    task_executor: TaskExecutor,
    web3: Arc<web3::Web3<T>>,
    abi: Arc<ethabi::Contract>,
    message_id: H256,
    token_id: U256,
) where
    T: web3::Transport + Send + Sync + 'static,
    T::Out: Send,
{
    let args = (message_id,);
    let eth_validator_private_key = config.eth_validator_private_key.clone();
    let contract_address = get_bridge_address(token_id.low_u32(), config);
    let eth_gas_price = config.eth_gas_price;
    let eth_gas = config.eth_gas;
    let data = ethereum_transactions::build_transaction_data(&abi, "confirmTransfer", args);
    let fut = web3.eth().transaction_count(config.eth_validator_address, None)
        .and_then(move |nonce| {
            let tx = ethereum_transactions::build(eth_validator_private_key, contract_address, nonce, AMOUNT, eth_gas_price, eth_gas, data);
            log::debug!("raw confirmTransfer: {:?}", tx);
            web3.eth().send_raw_transaction(Bytes::from(tx))
                .then(move |res| {
                    match res {
                        Ok(tx_res) => {
                            log::info!("[ethereum] called confirmTransfer({:?}), nonce: {:?}, result: {:?}",
                                       args.0, nonce, tx_res)
                        },
                        Err(err) => {
                            log::info!("[ethereum] can not send confirmTransfer({:?}), nonce: {:?}, reason: {:?}",
                                       args.0, nonce, err)
                        }
                    }

                    Ok(())
                })
        })
        .or_else(|e| {
            log::warn!("can not get nonce: {:?}", e);
            Ok(())
        });
    task_executor.spawn(fut);
}

fn handle_sub_burned_message<T>(
    config: &Config,
    task_executor: TaskExecutor,
    web3: Arc<web3::Web3<T>>,
    abi: Arc<ethabi::Contract>,
    message_id: H256,
    token_id: U256,
) where
    T: web3::Transport + Send + Sync + 'static,
    T::Out: Send,
{
    let args = (message_id,);
    let eth_validator_private_key = config.eth_validator_private_key.clone();
    let contract_address = get_bridge_address(token_id.low_u32(), config);
    let eth_gas_price = config.eth_gas_price;
    let eth_gas = config.eth_gas;
    let data = ethereum_transactions::build_transaction_data(&abi, "confirmWithdrawTransfer", args);
    let fut = web3
        .eth()
        .transaction_count(config.eth_validator_address, None)
        .and_then(move |nonce| {
            let tx = ethereum_transactions::build(
                eth_validator_private_key,
                contract_address,
                nonce,
                AMOUNT,
                eth_gas_price,
                eth_gas,
                data,
            );
            log::debug!("raw confirmTransfer: {:?}", tx);
            web3.eth()
                .send_raw_transaction(Bytes::from(tx))
                .then(move |res| {
                    match res {
                        Ok(tx_res) => log::info!(
                            "[ethereum] called confirmBurn({:?}), nonce: {:?}, result: {:?}",
                            args.0,
                            nonce,
                            tx_res
                        ),
                        Err(err) => log::info!(
                            "[ethereum] can not send confirmBurn({:?}), nonce: {:?}, reason: {:?}",
                            args.0,
                            nonce,
                            err
                        ),
                    }

                    Ok(())
                })
        })
        .or_else(|e| {
            log::warn!("can not get nonce: {:?}", e);
            Ok(())
        });
    task_executor.spawn(fut);
}

fn handle_sub_cancellation_confirmed_message<T>(
    config: &Config,
    task_executor: TaskExecutor,
    web3: Arc<web3::Web3<T>>,
    abi: Arc<ethabi::Contract>,
    message_id: H256,
    token_id: U256,
) where
    T: web3::Transport + Send + Sync + 'static,
    T::Out: Send,
{
    let args = (message_id,);
    let eth_validator_private_key = config.eth_validator_private_key.clone();
    let contract_address = get_bridge_address(token_id.low_u32(), config);
    let eth_gas_price = config.eth_gas_price;
    let eth_gas = config.eth_gas;
    let data = ethereum_transactions::build_transaction_data(&abi, "confirmCancelTransfer", args);
    let fut = web3.eth().transaction_count(config.eth_validator_address, None)
        .and_then(move |nonce| {
            let tx = ethereum_transactions::build(eth_validator_private_key, contract_address, nonce, AMOUNT, eth_gas_price, eth_gas, data);
            log::debug!("raw confirmCancel: {:?}", tx);
            web3.eth().send_raw_transaction(Bytes::from(tx))
                .then(move |res| {
                    match res {
                        Ok(tx_res) => {
                            log::info!("[ethereum] called confirmCancel({:?}), nonce: {:?}, result: {:?}",
                                       args.0, nonce, tx_res)
                        },
                        Err(err) => {
                            log::info!("[ethereum] can not send confirmCancel({:?}), nonce: {:?}, reason: {:?}",
                                       args.0, nonce, err)
                        }
                    }

                    Ok(())
                })
        })
        .or_else(|e| {
            log::warn!("can not get nonce: {:?}", e);
            Ok(())
        });
    task_executor.spawn(fut);
}

fn handle_sub_account_paused_message<T>(
    config: &Config,
    task_executor: TaskExecutor,
    web3: Arc<web3::Web3<T>>,
    abi: Arc<ethabi::Contract>,
    message_id: H256,
    sub_address: H256,
    token_id: U256,
) where
    T: web3::Transport + Send + Sync + 'static,
    T::Out: Send,
{
    let args = (sub_address,);
    let eth_validator_private_key = config.eth_validator_private_key.clone();
    let contract_address = get_bridge_address(token_id.low_u32(), config);
    let eth_gas_price = config.eth_gas_price;
    let eth_gas = config.eth_gas;
    let data =
        ethereum_transactions::build_transaction_data(&abi, "setPausedStatusForGuestAddress", args);
    let fut = web3.eth().transaction_count(config.eth_validator_address, None)
        .and_then(move |nonce| {
            let tx = ethereum_transactions::build(eth_validator_private_key, contract_address, nonce, AMOUNT, eth_gas_price, eth_gas, data);
            log::debug!("raw setPausedStatusForGuestAddress: {:?}", tx);
            web3.eth().send_raw_transaction(Bytes::from(tx))
                .then(move |res| {
                    match res {
                        Ok(tx_res) => {
                            log::info!("[ethereum] called setPausedStatusForGuestAddress({:?}), message_id: {:?}, nonce: {:?}, result: {:?}",
                                       args.0, message_id, nonce, tx_res)
                        },
                        Err(err) => {
                            log::info!("[ethereum] can not send setPausedStatusForGuestAddress({:?}), message_id: {:?}, nonce: {:?}, reason: {:?}",
                                       args.0, message_id, nonce, err)
                        }
                    }

                    Ok(())
                })
        })
        .or_else(|e| {
            log::warn!("can not get nonce: {:?}", e);
            Ok(())
        });
    task_executor.spawn(fut);
}

fn handle_sub_account_resumed_message<T>(
    config: &Config,
    task_executor: TaskExecutor,
    web3: Arc<web3::Web3<T>>,
    abi: Arc<ethabi::Contract>,
    message_id: H256,
    sub_address: H256,
    token_id: U256,
) where
    T: web3::Transport + Send + Sync + 'static,
    T::Out: Send,
{
    let args = (sub_address,);
    let eth_validator_private_key = config.eth_validator_private_key.clone();
    let contract_address = get_bridge_address(token_id.low_u32(), config);
    let eth_gas_price = config.eth_gas_price;
    let eth_gas = config.eth_gas;
    let data = ethereum_transactions::build_transaction_data(
        &abi,
        "setResumedStatusForGuestAddress",
        args,
    );
    let fut = web3.eth().transaction_count(config.eth_validator_address, None)
        .and_then(move |nonce| {
            let tx = ethereum_transactions::build(eth_validator_private_key, contract_address, nonce, AMOUNT, eth_gas_price, eth_gas, data);
            log::debug!("raw setResumedStatusForGuestAddress: {:?}", tx);
            web3.eth().send_raw_transaction(Bytes::from(tx))
                .then(move |res| {
                    match res {
                        Ok(tx_res) => {
                            log::info!("[ethereum] called setResumedStatusForGuestAddress({:?}), message_id: {:?}, nonce: {:?}, result: {:?}",
                                       args.0, message_id, nonce, tx_res)
                        },
                        Err(err) => {
                            log::info!("[ethereum] can not send setResumedStatusForGuestAddress({:?}), message_id: {:?}, nonce: {:?}, reason: {:?}",
                                       args.0, message_id, nonce, err)
                        }
                    }

                    Ok(())
                })
        })
        .or_else(|e| {
            log::warn!("can not get nonce: {:?}", e);
            Ok(())
        });
    task_executor.spawn(fut);
}

fn get_contract_abi(tokens: HashMap<u32, Token>, token: u32) -> Arc<ethabi::Contract> {
    tokens
        .get(&token)
        .expect("Could not get token by id")
        .abi
        .clone()
}

fn get_bridge_address(token: u32, config: &Config) -> Address {
    match token {
        0 => config.dai_bridge_address,
        1 => config.cdai_bridge_address,
        2 => config.usdt_bridge_address,
        3 => config.usdc_bridge_address,
        _ => unreachable!(),
    }
}
fn get_token_id_by_address(token: H160, config: &Config) -> u32 {
    let Config {
        dai_contract_address,
        cdai_contract_address,
        usdt_contract_address,
        usdc_contract_address,
        ..
    } = config;
    match token {
        t if t == *dai_contract_address => 0,
        t if t == *cdai_contract_address => 1,
        t if t == *usdt_contract_address => 2,
        t if t == *usdc_contract_address => 3,
        _ => unreachable!(),
    }
}

/// Allocate contracts abi once in a runtime
fn init_abi(tokens: Vec<&[u8]>, config: &Config) -> HashMap<u32, Token> {
    tokens
        .iter()
        .enumerate()
        .map(|(i, t)| (i as u32, construct_token(*t, config)))
        .collect()
}

// used only here ^
fn construct_token(symbol: &[u8], config: &Config) -> Token {
    let abi = get_abi_file(symbol);
    let abi = ethabi::Contract::load(abi.to_vec().as_slice()).expect("can not read ABI");
    let abi = Arc::new(abi);
    let address = get_contract_address(symbol, config);
    let symbol = String::from_utf8(symbol.to_vec()).expect("could not convert symbol to String");
    Token {
        symbol,
        address,
        abi,
    }
}
// used only here ^
fn get_contract_address(symbol: &[u8], config: &Config) -> Address {
    match symbol {
        b"DAI" => config.dai_contract_address,
        b"cDAI" => config.cdai_contract_address,
        b"USDT" => config.usdt_contract_address,
        b"USDC" => config.usdc_contract_address,
        _ => unreachable!(),
    }
}
fn get_abi_file(token: &[u8]) -> &'static [u8] {
    match token {
        b"DAI" => include_bytes!("../res/contracts/DAIContract.json"),
        b"cDAI" => include_bytes!("../res/contracts/cDAIContract.json"),
        b"USDT" => include_bytes!("../res/contracts/USDTContract.json"),
        b"USDC" => include_bytes!("../res/contracts/USDCContract.json"),
        _ => unreachable!(),
    }
}
