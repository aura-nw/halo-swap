use cosmwasm_schema::cw_serde;
use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{ Item, Map};
use halo_pair::asset::AssetInfoRaw;
use halo_pair::pair::PairInfoRaw;


pub const CONFIG: Item<Config> = Item::new("config");
pub const TMP_PAIR_INFO: Item<TmpPairInfo> = Item::new("tmp_pair_info");
pub const PAIRS: Map<&[u8], PairInfoRaw> = Map::new("pair_info");

// settings for pagination
pub const MAX_LIMIT: u32 = 30;
pub const DEFAULT_LIMIT: u32 = 10;

#[cw_serde] 
pub struct Config {
    pub owner: CanonicalAddr,
    pub pair_code_id: u64,
    pub token_code_id: u64,
}

#[cw_serde]
pub struct TmpPairInfo {
    pub pair_key: Vec<u8>,
    pub asset_infos: [AssetInfoRaw; 2],
    pub asset_decimals: [u8; 2],
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub pair_code_id: u64,
    pub token_code_id: u64,
}

