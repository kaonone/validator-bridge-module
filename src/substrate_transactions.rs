use node_runtime::Balance;
use primitives::{
    crypto::{AccountId32, Pair},
    sr25519,
};
use substrate_api_client::{
    compose_extrinsic, extrinsic::xt_primitives::UncheckedExtrinsicV4, Api, XtStatus,
};

pub fn mint(
    sub_api_url: String,
    signer_mnemonic_phrase: String,
    message_id: primitives::H256,
    from: primitives::H160,
    to: AccountId32,
    token_id: u32,
    amount: u128,
) {
    let sub_api = Api::new(sub_api_url).set_signer(get_sr25519_pair(&signer_mnemonic_phrase));
    log::debug!("extrinsic input: module: Bridge, extrinsic: multi_signed_mint, from:{:?}, to:{:?}, token_id:{:?}, amount:{:?}", from, to, token_id, amount);
    let ext: UncheckedExtrinsicV4<_> = compose_extrinsic!(
        sub_api.clone(),
        "Bridge",
        "multi_signed_mint",
        message_id,
        from,
        GenericAddress::from(to),
        Compact(token_id),
        Compact(amount)
    );
    let ext_hexed = ext.hex_encode();
    log::debug!("extrinsic: {:?}", ext);
    //send and watch extrinsic until finalized
    let tx_hash = sub_api.send_extrinsic(ext_hexed, XtStatus::Finalized);
    match tx_hash {
        Ok(result) => match result {
            Some(h) => log::info!("multi_signed_mint successdul, tx hash: {:?}", h),
            None => log::error!("multi_signed_mint failed, grep for 'substrate_api_client] response is probably failed'"),
        },
        Err(e) => log::error!("multi_signed_mint failed, error: {:?}", e),
    };
}

pub fn approve_transfer(
    sub_api_url: String,
    signer_mnemonic_phrase: String,
    message_id: primitives::H256,
) {
    let sub_api = Api::new(sub_api_url).set_signer(get_sr25519_pair(&signer_mnemonic_phrase));
    let ext = compose_extrinsic!(sub_api, "Bridge", "approve_transfer", message_id);
    log::debug!("extrinsic: {:?}", ext);
    //send and watch extrinsic until finalized
    let tx_hash = sub_api.send_extrinsic(ext.hex_encode(), XtStatus::Finalized);
    match tx_hash {
        Ok(result) => match result {
            Some(h) => log::info!("approve_transfer successdul, tx hash: {:?}", h),
            None => log::error!("approve_transfer failed, grep for 'substrate_api_client] response is probably failed'"),
        },
        Err(e) => log::error!("approve_transfer failed, error: {:?}", e),
    };
}

pub fn cancel_transfer(
    sub_api_url: String,
    signer_mnemonic_phrase: String,
    message_id: primitives::H256,
) {
    let sub_api = Api::new(sub_api_url).set_signer(get_sr25519_pair(&signer_mnemonic_phrase));
    let ext = compose_extrinsic!(sub_api, "Bridge", "cancel_transfer", message_id);
    log::debug!("extrinsic: {:?}", ext);
    //send and watch extrinsic until finalized
    let tx_hash = sub_api.send_extrinsic(ext.hex_encode(), XtStatus::Finalized);
    match tx_hash {
        Ok(result) => match result {
            Some(h) => log::info!("cancel_transfer successdul, tx hash: {:?}", h),
            None => log::error!("cancel_transfer failed, grep for 'substrate_api_client] response is probably failed'"),
        },
        Err(e) => log::error!("cancel_transfer failed, error: {:?}", e),
    };
}

pub fn confirm_transfer(
    sub_api_url: String,
    signer_mnemonic_phrase: String,
    message_id: primitives::H256,
) {
    let sub_api = Api::new(sub_api_url).set_signer(get_sr25519_pair(&signer_mnemonic_phrase));
    let ext = compose_extrinsic!(sub_api, "Bridge", "confirm_transfer", message_id);
    log::debug!("extrinsic: {:?}", ext);
    //send and watch extrinsic until finalized
    let tx_hash = sub_api.send_extrinsic(ext.hex_encode(), XtStatus::Finalized);
    match tx_hash {
        Ok(result) => match result {
            Some(h) => log::info!("confirm_transfer successdul, tx hash: {:?}", h),
            None => log::error!("confirm_transfer failed, grep for 'substrate_api_client] response is probably failed'"),
        },
        Err(e) => log::error!("confirm_transfer failed, error: {:?}", e),
    };
}

pub fn pause_bridge(sub_api_url: String, signer_mnemonic_phrase: String) {
    let sub_api = Api::new(sub_api_url).set_signer(get_sr25519_pair(&signer_mnemonic_phrase));
    let ext = compose_extrinsic!(sub_api, "Bridge", "pause_bridge");
    log::debug!("extrinsic: {:?}", ext);
    //send and watch extrinsic until finalized
    let tx_hash = sub_api.send_extrinsic(ext.hex_encode(), XtStatus::Finalized);
    match tx_hash {
        Ok(result) => match result {
            Some(h) => log::info!("pause_bridge successdul, tx hash: {:?}", h),
            None => log::error!(
                "pause_bridge failed, grep for 'substrate_api_client] response is probably failed'"
            ),
        },
        Err(e) => log::error!("pause_bridge failed, error: {:?}", e),
    };
}

pub fn resume_bridge(sub_api_url: String, signer_mnemonic_phrase: String) {
    let sub_api = Api::new(sub_api_url).set_signer(get_sr25519_pair(&signer_mnemonic_phrase));
    let ext = compose_extrinsic!(sub_api, "Bridge", "resume_bridge");
    log::debug!("extrinsic: {:?}", ext);
    //send and watch extrinsic until finalized
    let tx_hash = sub_api.send_extrinsic(ext.hex_encode(), XtStatus::Finalized);
    match tx_hash {
        Ok(result) => match result {
            Some(h) => log::info!("resume_bridge successdul, tx hash: {:?}", h),
            None => log::error!("resume_bridge failed, grep for 'substrate_api_client] response is probably failed'"),
        },
        Err(e) => log::error!("resume_bridge failed, error: {:?}", e),
    };
}

pub fn update_limits(
    sub_api_url: String,
    signer_mnemonic_phrase: String,
    min_guest_transaction_value: u128,
    max_guest_transaction_value: u128,
    day_guest_max_limit: u128,
    day_guest_max_limit_for_one_address: u128,
    max_guest_pending_transaction_limit: u128,
) {
    let sub_api = Api::new(sub_api_url).set_signer(get_sr25519_pair(&signer_mnemonic_phrase));
    let ext = compose_extrinsic!(
        sub_api,
        "Bridge",
        "update_limits",
        min_guest_transaction_value,
        max_guest_transaction_value,
        day_guest_max_limit,
        day_guest_max_limit_for_one_address,
        max_guest_pending_transaction_limit
    );
    log::debug!("extrinsic: {:?}", ext);
    //send and watch extrinsic until finalized
    let tx_hash = sub_api.send_extrinsic(ext.hex_encode(), XtStatus::Finalized);
    match tx_hash {
        Ok(result) => match result {
            Some(h) => log::info!("update_limits successdul, tx hash: {:?}", h),
            None => log::error!("update_limits failed, grep for 'substrate_api_client] response is probably failed'"),
        },
        Err(e) => log::error!("update_limits failed, error: {:?}", e),
    };
}

pub fn update_validator_list(
    sub_api_url: String,
    signer_mnemonic_phrase: String,
    message_id: primitives::H256,
    new_how_many_validators_decide: u64,
    new_validators: Vec<AccountId32>,
) {
    let sub_api = Api::new(sub_api_url).set_signer(get_sr25519_pair(&signer_mnemonic_phrase));
    let ext = compose_extrinsic!(
        sub_api,
        "Bridge",
        "update_validator_list",
        message_id,
        new_how_many_validators_decide,
        new_validators
    );
    log::debug!("extrinsic: {:?}", ext);
    //send and watch extrinsic until finalized
    let tx_hash = sub_api.send_extrinsic(ext.hex_encode(), XtStatus::Finalized);
    match tx_hash {
        Ok(result) => match result {
            Some(h) => log::info!("update_validator_list successdul, tx hash: {:?}", h),
            None => log::error!("update_validator_list failed, grep for 'substrate_api_client] response is probably failed'"),
        },
        Err(e) => log::error!("update_validator_list failed, error: {:?}", e),
    };
}

pub fn record_price(
    sub_api_url: String,
    signer_mnemonic_phrase: String,
    token: Vec<u8>,
    price: Balance,
) {
    let sub_api = Api::new(sub_api_url).set_signer(get_sr25519_pair(&signer_mnemonic_phrase));
    let ext = compose_extrinsic!(sub_api, "Oracle", "record_price", token, price);
    log::debug!("extrinsic: {:?}", ext);
    //send and watch extrinsic until finalized
    let tx_hash = sub_api.send_extrinsic(ext.hex_encode(), XtStatus::Finalized);
    match tx_hash {
        Ok(result) => match result {
            Some(h) => log::info!("record_price successdul, tx hash: {:?}", h),
            None => log::error!(
                "record_price failed, grep for 'substrate_api_client] response is probably failed'"
            ),
        },
        Err(e) => log::error!("record_price failed, error: {:?}", e),
    };
}

pub fn get_sr25519_pair(signer_mnemonic_phrase: &str) -> sr25519::Pair {
    sr25519::Pair::from_phrase(signer_mnemonic_phrase, None)
        .expect("invalid mnemonic phrase")
        .0
}
