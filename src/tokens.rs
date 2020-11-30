use cosmwasm_std::{
    Binary, CosmosMsg, HumanAddr, StdResult, Storage, Uint128, WasmMsg,
};

use crate::state::{ config_read };

pub fn mint<S: Storage>(store: &S, amount: Uint128, account: HumanAddr) -> StdResult<CosmosMsg> {
    let constants = config_read(store).load()?;

    return Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: constants.token_contract.address,
        callback_code_hash: constants.token_contract.code_hash,
        msg: Binary(
            format!(
                r#"{{"mint": {{"address":"{}", "amount":"{}"}} }}"#,
                account.to_string(),
                amount.to_string()
            )
            .as_bytes()
            .to_vec(),
        ),
        send: vec![],
    }));
}
