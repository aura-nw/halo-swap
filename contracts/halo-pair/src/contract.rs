#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::pair::{ PairInfoRaw, PAIR_INFO, COMMISSION_RATE };
use crate::asset::{Asset, AssetInfo, query_token_info};

use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary, Decimal,
    CanonicalAddr, SubMsg, WasmMsg, ReplyOn, Uint128, CosmosMsg, StdError, Addr
};
use halo_token::msg::InstantiateMsg as TokenInstantiateMsg;
use cw20::{MinterResponse, Cw20ExecuteMsg};
use integer_sqrt::IntegerSquareRoot;
use std::cmp::Ordering;
use std::str::FromStr;
use cosmwasm_bignumber::{Decimal256, Uint256};

const CONTRACT_NAME: &str = "crates.io:halo-pair";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // create pair info from msg
    let pair_info: &PairInfoRaw = &PairInfoRaw {
        contract_addr: deps.api.addr_canonicalize(env.contract.address.as_str())?,
        liquidity_token: CanonicalAddr::from(vec![]),   // will be set later, after token contract is created
        asset_infos: [
            msg.asset_infos[0].to_raw(deps.api)?,
            msg.asset_infos[1].to_raw(deps.api)?,
        ],
        asset_decimals: msg.asset_decimals,
    };

    // store pair info
    PAIR_INFO.save(deps.storage, pair_info)?;

    Ok(Response::new().add_submessage(SubMsg {
        // We instantiate new LP token here
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: msg.token_code_id,
            msg: to_binary(&TokenInstantiateMsg {
                name: "Halo LP Token".to_string(),          // TODO: Modify this name to ensure that it distinguishes between different pairs
                symbol: "uLP".to_string(),                  // TODO: Modify this symbol to ensure that it distinguishes between different pairs
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
            })?,
            funds: vec![],
            label: "Halo LP Token".to_string(),
        }.into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
    
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ProvideLiquidity {
            assets,
            slippage_tolerance,
            receiver,
        } => provide_liquidity(deps, env, info, assets, slippage_tolerance, receiver),
        // Skip 'Receive' message temporarily
        // // this message is ONLY used when swapping from a cw20 token to another cw20 token
        // ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        // // this message is ONLY used when swapping from native token to cw20 token
        ExecuteMsg::Swap {
            offer_asset,
            belief_price,
            max_spread,
            to,
        } => {
            // if the offer asset is not native token, return error
            if !offer_asset.is_native_token() {
                return Err(ContractError::Unauthorized {});
            }

            // verify the 'to' address
            let to_addr = if let Some(to_addr) = to {
                Some(deps.api.addr_validate(&to_addr)?)
            } else {
                None
            };

            swap(
                deps,
                env,
                info.clone(),
                info.sender,
                offer_asset,
                belief_price,
                max_spread,
                to_addr,
            )
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::NativeTokenDecimals { _denom } => {
            // to_binary(&query_native_token_decimal(deps, denom)?)
            // return default response
            Ok(Binary::from(b"0".to_vec()))
        }
    }
}

pub fn provide_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: [Asset; 2],
    slippage_tolerance: Option<Decimal>,
    receiver: Option<String>,
) -> Result<Response, ContractError> {
    for asset in assets.iter() {
        // check the balance of native token is sent with the message
        asset.assert_sent_native_token_balance(&info)?;
    }

    // get information of the pair
    let pair_info: PairInfoRaw = PAIR_INFO.load(deps.storage)?;

    // query the information of the pair of assets
    let mut pools: [Asset; 2] = pair_info.query_pools(&deps.querier, deps.api, env.contract.address.clone())?;

    // get the amount of assets that user deposited after checking the assets is same as the assets in pair
    let deposits: [Uint128; 2] = [
        assets
            .iter()
            .find(|a| a.info.equal(&pools[0].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
        assets
            .iter()
            .find(|a| a.info.equal(&pools[1].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
    ];

    // If the asset is a token, the value of pools[i] is correct. But we must take the token from the user.
    // If the asset is a native token, the amount of native token is already sent with the message to the pool.
    // So we must subtract that ammount of native token from the pools[i].
    // pools[] will be used to calculate the amount of LP token to mint after.
    let mut messages: Vec<CosmosMsg> = vec![];
    for (i, pool) in pools.iter_mut().enumerate() {
        // If the asset 'pool' is a token, then we need to execute TransferFrom msg to receive funds
        // User must approve the pool contract to transfer the token before calling this function
        if let AssetInfo::Token { contract_addr, .. } = &pool.info {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: deposits[i],
                })?,
                funds: vec![],
            }));
        } else {
            // If the asset 'pool' is native token, balance is already increased
            // To calculated properly we should subtract user deposit from the pool
            pool.amount = pool.amount.checked_sub(deposits[i])?;
        }
    }

    // if the user provides the slippage tolerance, we should check it
    assert_slippage_tolerance(&slippage_tolerance, &deposits, &pools)?;

    // get the address of the LP token
    let liquidity_token = deps.api.addr_humanize(&pair_info.liquidity_token)?;

    // get total supply of the LP token
    let total_share = query_token_info(&deps.querier, liquidity_token)?.total_supply;

    // calculate the amount of LP token is minted to the user
    let share = if total_share == Uint128::zero() {
        // if the total supply of the LP token is zero, Initial share = collateral amount
        // hoanm: EQUATION - LP = \sqrt{A * B}
        Uint128::from((deposits[0].u128() * deposits[1].u128()).integer_sqrt())
    } else {
        // hoanm: update these equations by using the formula of Uniswap V2
        // min(1, 2)
        // 1. sqrt(deposit_0 * exchange_rate_0_to_1 * deposit_0) * (total_share / sqrt(pool_0 * pool_1))
        // == deposit_0 * total_share / pool_0
        // 2. sqrt(deposit_1 * exchange_rate_1_to_0 * deposit_1) * (total_share / sqrt(pool_1 * pool_1))
        // == deposit_1 * total_share / pool_1
        std::cmp::min(
            deposits[0].multiply_ratio(total_share, pools[0].amount),
            deposits[1].multiply_ratio(total_share, pools[1].amount),
        )
    };

    // prevent providing free token (hoanm: is this necessary?)
    if share.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // mint LP token to sender
    // if the user provides the receiver, mint LP token to the receiver else mint to the sender
    let receiver = receiver.unwrap_or_else(|| info.sender.to_string());

    // mint amount of 'share' LP token to the receiver
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps
            .api
            .addr_humanize(&pair_info.liquidity_token)?
            .to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: receiver.to_string(),
            amount: share,
        })?,
        funds: vec![],
    }));

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "provide_liquidity"),
        ("sender", info.sender.as_str()),
        ("receiver", receiver.as_str()),
        ("assets", &format!("{}, {}", assets[0], assets[1])),
        ("share", &share.to_string()),
    ]))
}

fn assert_slippage_tolerance(
    slippage_tolerance: &Option<Decimal>,
    deposits: &[Uint128; 2],
    pools: &[Asset; 2],
) -> Result<(), ContractError> {
    if let Some(slippage_tolerance) = *slippage_tolerance {
        let slippage_tolerance: Decimal256 = slippage_tolerance.into();
        // the slippage tolerance cannot be greater than 100%
        if slippage_tolerance > Decimal256::one() {
            return Err(StdError::generic_err("slippage_tolerance cannot bigger than 1").into());
        }

        let one_minus_slippage_tolerance = Decimal256::one() - slippage_tolerance;
        let deposits: [Uint256; 2] = [deposits[0].into(), deposits[1].into()];
        let pools: [Uint256; 2] = [pools[0].amount.into(), pools[1].amount.into()];

        // Ensure each prices are not dropped as much as slippage tolerance rate
        // hoanm: EQUATION - \frac{A}{B} * (1-ST) > \frac{R_A}{R_B} \parallel \frac{B}{A} * (1-ST) > \frac{R_B}{R_A}
        if Decimal256::from_ratio(deposits[0], deposits[1]) * one_minus_slippage_tolerance
            > Decimal256::from_ratio(pools[0], pools[1])
            || Decimal256::from_ratio(deposits[1], deposits[0]) * one_minus_slippage_tolerance
                > Decimal256::from_ratio(pools[1], pools[0])
        {
            return Err(ContractError::MaxSlippageAssertion {});
        }
    }

    Ok(())
}

// Function swap from native token to token
pub fn swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    offer_asset: Asset,
    belief_price: Option<Decimal>,
    max_spread: Option<Decimal>,
    to: Option<Addr>,
) -> Result<Response, ContractError> {
    // check amount of asset if it is native token
    offer_asset.assert_sent_native_token_balance(&info)?;

    // load information of pair
    // pair_info contains:
    // asset_infos - information of 2 assets in the pair (token address or native token)
    // contract_addr - the address of the pair contract
    // liquidity_token - the address of the LP token corresponding to the pair
    // asset_decimals - the decimals of 2 assets in the pair
    let pair_info: PairInfoRaw = PAIR_INFO.load(deps.storage)?;

    // query pools - contains the information of 2 assets in the pair
    let pools: [Asset; 2] = pair_info.query_pools(&deps.querier, deps.api, env.contract.address)?;

    let offer_pool: Asset;
    let ask_pool: Asset;

    let offer_decimal: u8;
    let ask_decimal: u8;

    // This function is called when user swaps from native token to cw20 token.
    // The native token will be sent with the message.
    // So, the amount of offer_pool was increased by the amount of offer_asset before these below lines of code 
    // If the asset balance is already increased
    // To calculated properly we should subtract user deposit from the pool
    // decide in the pair, which pool is offer_pool and which is ask_pool
    if offer_asset.info.equal(&pools[0].info) {
        offer_pool = Asset {
            amount: pools[0].amount.checked_sub(offer_asset.amount)?,
            info: pools[0].info.clone(),
        };
        ask_pool = pools[1].clone();

        offer_decimal = pair_info.asset_decimals[0];
        ask_decimal = pair_info.asset_decimals[1];
    } else if offer_asset.info.equal(&pools[1].info) {
        offer_pool = Asset {
            amount: pools[1].amount.checked_sub(offer_asset.amount)?,
            info: pools[1].info.clone(),
        };
        ask_pool = pools[0].clone();

        offer_decimal = pair_info.asset_decimals[1];
        ask_decimal = pair_info.asset_decimals[0];
    } else {
        return Err(ContractError::AssetMismatch {});
    }

    // calculate the offer_amount
    let offer_amount = offer_asset.amount;

    // calculate the return amount, the spread amount and the commission amount using compute_swap function
    // the return_amount is the amount of ask_pool that the user will receive
    let (return_amount, spread_amount, commission_amount) = compute_swap(offer_pool.amount, ask_pool.amount, offer_amount);

    let return_asset = Asset {
        info: ask_pool.info.clone(),
        amount: return_amount,
    };

    // check max spread limit if exist
    assert_max_spread(
        belief_price,
        max_spread,
        offer_asset.clone(),
        return_asset.clone(),
        spread_amount,
        offer_decimal,
        ask_decimal,
    )?;

    // The pair call Swap function and address 'to' will be the current pair contract address until the last operation
    let receiver = to.unwrap_or_else(|| sender.clone());

    let mut messages: Vec<CosmosMsg> = vec![];
    if !return_amount.is_zero() {
        messages.push(return_asset.into_msg(receiver.clone())?);
    }

    // 1. send collateral token from the contract to a user
    // 2. send inactive commission to collector
    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "swap"),
        ("sender", sender.as_str()),
        ("receiver", receiver.as_str()),
        ("offer_asset", &offer_asset.info.to_string()),
        ("ask_asset", &ask_pool.info.to_string()),
        ("offer_amount", &offer_amount.to_string()),
        ("return_amount", &return_amount.to_string()),
        ("spread_amount", &spread_amount.to_string()),
        ("commission_amount", &commission_amount.to_string()),
    ]))

}

// User want to swap from 'offer' to 'ask'
// Calculate the expected return_amount, spread_amount and commission_amount based on the formula
// return_amount = offer_amount * (1 - spread) * ask_pool / (offer_pool + offer_amount)
fn  compute_swap(
    offer_pool: Uint128,
    ask_pool: Uint128,
    offer_amount: Uint128,
) -> (Uint128, Uint128, Uint128) {
    let offer_pool: Uint256 = Uint256::from(offer_pool);
    let ask_pool: Uint256 = ask_pool.into();
    let offer_amount: Uint256 = offer_amount.into();

    // Commission rate OR Fee amount for framework
    let commission_rate = Decimal256::from_str(COMMISSION_RATE).unwrap();

    // offer => ask
    // hoanm: EQUATION - B = (R_B - \frac{K}{R_A + A}) * (1 - F)
    // ask_amount = (ask_pool - cp / (offer_pool + offer_amount)) * (1 - commission_rate)

    // cp (constant product) is K  in the EQUATION
    let cp: Uint256 = offer_pool * ask_pool;

    // calculate the ask_amount without commission
    let return_amount: Uint256 = (Decimal256::from_uint256(ask_pool)
        - Decimal256::from_ratio(cp, offer_pool + offer_amount))
        * Uint256::one();

    // calculate the spread_amount
    // hoanm: EQUATION - SPREAD = (A * \frac{R_B}{R_A}) - B
    let spread_amount: Uint256 =
        (offer_amount * Decimal256::from_ratio(ask_pool, offer_pool)) - return_amount;

    // calculate the commission_amount
    let commission_amount: Uint256 = return_amount * commission_rate;

    // commission will be absorbed to pool and the currency will be the same as the ask currency
    let return_amount: Uint256 = return_amount - commission_amount;
    (
        return_amount.into(),
        spread_amount.into(),
        commission_amount.into(),
    )
}

/// If `belief_price` and `max_spread` both are given,
/// we compute new spread else we just use auraswap
/// spread to check `max_spread`
pub fn assert_max_spread(
    belief_price: Option<Decimal>,
    max_spread: Option<Decimal>,
    offer_asset: Asset,
    return_asset: Asset,
    spread_amount: Uint128,
    offer_decimal: u8,
    return_decimal: u8,
) -> Result<(), ContractError> {
    let (offer_amount, return_amount, spread_amount): (Uint256, Uint256, Uint256) =
        match offer_decimal.cmp(&return_decimal) {
            Ordering::Greater => {
                let diff_decimal = 10u64.pow((offer_decimal - return_decimal).into());

                (
                    offer_asset.amount.into(),
                    return_asset
                        .amount
                        .checked_mul(Uint128::from(diff_decimal))?
                        .into(),
                    spread_amount
                        .checked_mul(Uint128::from(diff_decimal))?
                        .into(),
                )
            }
            Ordering::Less => {
                let diff_decimal = 10u64.pow((return_decimal - offer_decimal).into());

                (
                    offer_asset
                        .amount
                        .checked_mul(Uint128::from(diff_decimal))?
                        .into(),
                    return_asset.amount.into(),
                    spread_amount.into(),
                )
            }
            Ordering::Equal => (
                offer_asset.amount.into(),
                return_asset.amount.into(),
                spread_amount.into(),
            ),
        };

    if let (Some(max_spread), Some(belief_price)) = (max_spread, belief_price) {
        let belief_price: Decimal256 = belief_price.into();
        let max_spread: Decimal256 = max_spread.into();

        let expected_return = offer_amount / belief_price;
        let spread_amount = if expected_return > return_amount {
            expected_return - return_amount
        } else {
            Uint256::zero()
        };

        if return_amount < expected_return
            && Decimal256::from_ratio(spread_amount, expected_return) > max_spread
        {
            return Err(ContractError::MaxSpreadAssertion {});
        }
    } else if let Some(max_spread) = max_spread {
        let max_spread: Decimal256 = max_spread.into();
        if Decimal256::from_ratio(spread_amount, return_amount + spread_amount) > max_spread {
            return Err(ContractError::MaxSpreadAssertion {});
        }
    }

    Ok(())
}