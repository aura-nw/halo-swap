#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, to_binary, Addr, AllBalanceResponse, BalanceResponse, BankQuery, Coin, Decimal, QuerierWrapper,
    QueryRequest, WasmQuery, CanonicalAddr, SubMsg, WasmMsg, ReplyOn, from_binary, StdError
};
use halo_token::msg::InstantiateMsg as TokenInstantiateMsg;
use cw20::{MinterResponse, Cw20ReceiveMsg};

const CONTRACT_NAME: &str = "crates.io:halo-pair";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // // create pair info from msg
    // let pair_info: &PairInfoRaw = &PairInfoRaw {
    //     contract_addr: deps.api.addr_canonicalize(env.contract.address.as_str())?,
    //     liquidity_token: CanonicalAddr::from(vec![]),
    //     asset_infos: [
    //         msg.asset_infos[0].to_raw(deps.api)?,
    //         msg.asset_infos[1].to_raw(deps.api)?,
    //     ],
    //     asset_decimals: msg.asset_decimals,
    // };

    // // store pair info
    // PAIR_INFO.save(deps.storage, pair_info)?;

    // Ok(Response::new().add_submessage(SubMsg {
    //     // We instantiate new LP token here
    //     msg: WasmMsg::Instantiate {
    //         admin: None,
    //         code_id: msg.token_code_id,
    //         msg: to_binary(&TokenInstantiateMsg {
    //             name: "Halo LP Token".to_string(),          // TODO: Modify this name to ensure that it distinguishes between different pairs
    //             symbol: "uLP".to_string(),                  // TODO: Modify this symbol to ensure that it distinguishes between different pairs
    //             decimals: 6,
    //             initial_balances: vec![],
    //             mint: Some(MinterResponse {
    //                 minter: env.contract.address.to_string(),
    //                 cap: None,
    //             }),
    //         })?,
    //         funds: vec![],
    //         label: "Halo LP Token".to_string(),
    //     }.into(),
    //     gas_limit: None,
    //     id: INSTANTIATE_REPLY_ID,
    //     reply_on: ReplyOn::Success,
    // }))
    
    // return default response
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {

    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {

    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {

    todo!()
}

