use cosmwasm_schema::cw_serde;

use cosmwasm_std::{CanonicalAddr, Uint128};
use cw_storage_plus::Item;

use halo_pair::asset::AssetInfo;

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    pub auraswap_factory: CanonicalAddr,
}

#[cw_serde]
pub enum SwapOperation {
    AuraSwap {
        offer_asset_info: AssetInfo,
        ask_asset_info: AssetInfo,
    },
}

impl SwapOperation {
    pub fn get_target_asset_info(&self) -> AssetInfo {
        match self {
            SwapOperation::AuraSwap { ask_asset_info, .. } => ask_asset_info.clone(),
        }
    }
}

#[cw_serde]
pub enum Cw20HookMsg {
    ExecuteSwapOperations {
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        to: Option<String>,
    },
}