#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

use cw2::set_contract_version;
use cw20_base::contract::{create_accounts, execute as cw20_execute, query as cw20_query};
use cw20_base::msg::{ExecuteMsg, QueryMsg};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};
use cw20_base::ContractError;

use crate::msg::InstantiateMsg;

const CONTRACT_NAME: &str = "crates.io:halo-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // check valid token information
    msg.validate()?;

    // initiate balances for an addresses. Contains an address and the amount of tokens for that address.
    let total_supply = create_accounts(&mut deps, &msg.initial_balances)?;

    // we need ensure that the initial supply is not greater than the cap
    if let Some(cap) = msg.get_cap() {
        if total_supply > cap {
            return Err(ContractError::Std(StdError::generic_err(
                "Initial supply greater than cap",
            )));
        }
    }

    // create mint info from the mint value in the InstantiateMsg
    let mint = match msg.mint {
        Some(m) => Some(MinterData {
            minter: deps.api.addr_validate(&m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    // create token info from the InstantiateMsg
    let info = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply,
        mint,
    };

    // store the token info
    TOKEN_INFO.save(deps.storage, &info)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // we pass the execution to the cw20 base contract
    cw20_execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps, 
    env: Env, 
    msg: QueryMsg
) -> StdResult<Binary> {
    // we pass the query to the cw20 base contract
    cw20_query(deps, env, msg)
}

