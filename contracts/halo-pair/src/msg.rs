use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use cw20::Cw20ReceiveMsg;
use crate::asset::{ Asset, AssetInfo};

#[cw_serde]
pub struct InstantiateMsg {
    /// Asset infos: we need 2 assets to create a pair
    pub asset_infos: [AssetInfo; 2],
    /// Token contract code id for initialization
    pub token_code_id: u64,
    pub asset_decimals: [u8; 2],
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    /// ProvideLiquidity a user provides pool liquidity
    ProvideLiquidity {
        assets: [Asset; 2],
        slippage_tolerance: Option<Decimal>,
        receiver: Option<String>,
    },
    /// Swap an offer asset to the other
    Swap {
        offer_asset: Asset,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<String>,
    },
}

#[cw_serde]
pub enum MigrateMsg {}

#[cw_serde]
pub enum QueryMsg {
    NativeTokenDecimals {
        _denom: String,
    },
    Pair {},
    // Pool {},
    // Simulation { offer_asset: Asset },
    // ReverseSimulation { ask_asset: Asset },
}
