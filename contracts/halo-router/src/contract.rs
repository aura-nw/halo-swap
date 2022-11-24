use std::collections::HashMap;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Api, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError, CosmosMsg, QuerierWrapper, QueryRequest, WasmQuery, to_binary, Decimal, WasmMsg, Coin, Uint128, from_binary};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use halo_pair::asset::{AssetInfo, query_balance, query_token_balance, Asset};
use halo_pair::pair::PairInfo;
use halo_pair::msg::ExecuteMsg as PairExecuteMsg;
use halo_factory::msg::QueryMsg as FactoryQueryMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, SwapOperation, Cw20HookMsg};

const CONTRACT_NAME: &str = "crates.io:halo-router";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            auraswap_factory: deps.api.addr_canonicalize(&msg.auraswap_factory)?,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::ExecuteSwapOperations {
            operations,
            minimum_receive,
            to,
        } => {
            let api = deps.api;
            execute_swap_operations(
                deps,
                env,
                info.sender,
                operations,
                minimum_receive,
                optional_addr_validate(api, to)?,
            )
        }
        ExecuteMsg::ExecuteSwapOperation { operation, to } => {
            let api = deps.api;
            execute_swap_operation(
                deps,
                env,
                info,
                operation,
                optional_addr_validate(api, to)?.map(|v| v.to_string()),
            )
        }
        ExecuteMsg::AssertMinimumReceive {
            asset_info,
            prev_balance,
            minimum_receive,
            receiver,
        } => assert_minium_receive(
            deps.as_ref(),
            asset_info,
            prev_balance,
            minimum_receive,
            deps.api.addr_validate(&receiver)?,
        ),   
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        
    }
}

fn optional_addr_validate(api: &dyn Api, addr: Option<String>) -> StdResult<Option<Addr>> {
    let addr = if let Some(addr) = addr {
        Some(api.addr_validate(&addr)?)
    } else {
        None
    };

    Ok(addr)
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let sender = deps.api.addr_validate(&cw20_msg.sender)?;
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::ExecuteSwapOperations {
            operations,
            minimum_receive,
            to,
        } => {
            let api = deps.api;
            execute_swap_operations(
                deps,
                env,
                sender,
                operations,
                minimum_receive,
                optional_addr_validate(api, to)?,
            )
        }
    }
}

// The function will be used to begin the swap process.
// It's called by the user who wants to swap assets in FE.
// @param operations: the list of operations to be executed. Each operation is a pair of assets will be used to route the swap.
// // Maybe the router will be BE, so the user will not need to know the list of operations and the contract does not
pub fn execute_swap_operations(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    operations: Vec<SwapOperation>,
    minimum_receive: Option<Uint128>,
    to: Option<Addr>,
) -> StdResult<Response> {
    // the length of operations (number of swapping) must be greater than 0
    let operations_len = operations.len();
    if operations_len == 0 {
        return Err(StdError::generic_err("must provide operations"));
    }

    // Check if the operations sequence is valid
    assert_operations(&operations)?;

    // If the address 'to' is not provided, the default address 'to' is the sender
    let to = if let Some(to) = to { to } else { sender };

    // The target asset (ask asset) is the last asset in the last operation
    let target_asset_info = operations.last().unwrap().get_target_asset_info();

    
    let mut operation_index = 0;
    let mut messages: Vec<CosmosMsg> = operations
        .into_iter()
        .map(|op| {
            operation_index += 1;
            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: op,
                    to: if operation_index == operations_len {
                        Some(to.to_string())
                    } else {
                        None
                    },
                })?,
            }))
        })
        .collect::<StdResult<Vec<CosmosMsg>>>()?;

    // Execute minimum amount assertion
    if let Some(minimum_receive) = minimum_receive {
        let receiver_balance = target_asset_info.query_pool(&deps.querier, deps.api, to.clone())?;

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            funds: vec![],
            msg: to_binary(&ExecuteMsg::AssertMinimumReceive {
                asset_info: target_asset_info,
                prev_balance: receiver_balance,
                minimum_receive,
                receiver: to.to_string(),
            })?,
        }))
    }

    Ok(Response::new().add_messages(messages))
}

/// Execute swap operation
/// swap all offer asset to ask asset
/// This function is only called by this contract
/// @param operation <SwapOperation>: the pair to swap
/// @param to <Option<String>>: the receiver of ask asset
pub fn execute_swap_operation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operation: SwapOperation,
    to: Option<String>,
) -> StdResult<Response> {
    // if the caller is not this contract, return error
    if env.contract.address != info.sender {
        return Err(StdError::generic_err("unauthorized"));
    }

    let messages: Vec<CosmosMsg> = match operation {
        // asset_info is the address of asset if asset is token OR is the denom is asset is native token
        SwapOperation::AuraSwap {
            offer_asset_info,
            ask_asset_info,
        } => {
            // load config of auraswap factory
            let config: Config = CONFIG.load(deps.as_ref().storage)?;

            // check the address of auraswap factory
            let auraswap_factory = deps.api.addr_humanize(&config.auraswap_factory)?;

            // query the information of pair that user want to swap from auraswap factory:
            // asset_infos: two assets of pair
            // contract_addr: the address of pair contract
            // liquidity_token: the address of liquidity token corresponding to pair
            // asset_decimals: the decimal of two assets
            let pair_info: PairInfo = query_pair_info(
                &deps.querier,
                auraswap_factory,
                &[offer_asset_info.clone(), ask_asset_info],
            )?;

            // query the balance amount of offer in the contract
            let amount = match offer_asset_info.clone() {
                AssetInfo::NativeToken { denom } => {
                    query_balance(&deps.querier, env.contract.address, denom)?
                }
                AssetInfo::Token { contract_addr } => query_token_balance(
                    &deps.querier,
                    deps.api.addr_validate(contract_addr.as_str())?,
                    env.contract.address,
                )?,
            };
            let offer_asset: Asset = Asset {
                info: offer_asset_info,
                amount,
            };

            vec![asset_into_swap_msg(
                deps.as_ref(),
                Addr::unchecked(pair_info.contract_addr),
                offer_asset,
                None,
                to,
            )?]
        }
    };

    Ok(Response::new().add_messages(messages))
}

fn assert_minium_receive(
    deps: Deps,
    asset_info: AssetInfo,
    prev_balance: Uint128,
    minium_receive: Uint128,
    receiver: Addr,
) -> StdResult<Response> {
    let receiver_balance = asset_info.query_pool(&deps.querier, deps.api, receiver)?;
    let swap_amount = receiver_balance.checked_sub(prev_balance)?;

    if swap_amount < minium_receive {
        return Err(StdError::generic_err(format!(
            "assertion failed; minimum receive amount: {}, swap amount: {}",
            minium_receive, swap_amount
        )));
    }

    Ok(Response::default())
}

pub fn query_pair_info(
    querier: &QuerierWrapper,
    factory_contract: Addr,
    asset_infos: &[AssetInfo; 2],
) -> StdResult<PairInfo> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: factory_contract.to_string(),
        msg: to_binary(&FactoryQueryMsg::Pair {
            asset_infos: asset_infos.clone(),
        })?,
    }))
}

pub fn asset_into_swap_msg(
    _deps: Deps,
    pair_contract: Addr,
    offer_asset: Asset,
    max_spread: Option<Decimal>,
    to: Option<String>,
) -> StdResult<CosmosMsg> {
    match offer_asset.info.clone() {
        AssetInfo::NativeToken { denom } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pair_contract.to_string(),
            funds: vec![Coin {
                denom,
                amount: offer_asset.amount,
            }],
            msg: to_binary(&PairExecuteMsg::Swap {
                offer_asset,
                belief_price: None,
                max_spread,
                to,
            })?,
        })),
        AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: pair_contract.to_string(),
                amount: offer_asset.amount,
                msg: to_binary(&PairExecuteMsg::Swap {
                    offer_asset,
                    belief_price: None,
                    max_spread,
                    to,
                })?,
            })?,
        })),
    }
}

// This function is check if the operations in the router is executed samelessly
// The ask asset of the previous operation should be the offer asset of the next operation
// The last ask asset of this sequence should be 1 asset
fn assert_operations(operations: &[SwapOperation]) -> StdResult<()> {
    let mut ask_asset_map: HashMap<String, bool> = HashMap::new();
    for operation in operations.iter() {
        let (offer_asset, ask_asset) = match operation {
            SwapOperation::AuraSwap {
                offer_asset_info,
                ask_asset_info,
            } => (offer_asset_info.clone(), ask_asset_info.clone()),
        };

        ask_asset_map.remove(&offer_asset.to_string());
        ask_asset_map.insert(ask_asset.to_string(), true);
    }

    if ask_asset_map.keys().len() != 1 {
        return Err(StdError::generic_err(
            "invalid operations; multiple output token",
        ));
    }

    Ok(())
}