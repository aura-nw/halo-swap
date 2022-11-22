use cosmwasm_schema::cw_serde;
use halo_pair::asset::AssetInfo;

#[cw_serde]
pub struct InstantiateMsg {
    /// Pair contract code ID, which is used to
    pub pair_code_id: u64,
    pub token_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// UpdateConfig update relevant code IDs
    UpdateConfig {
        owner: Option<String>,
        token_code_id: Option<u64>,
        pair_code_id: Option<u64>,
    },
    /// CreatePair instantiates pair contract
    CreatePair {
        /// Asset infos
        asset_infos: [AssetInfo; 2],
    },
    AddNativeTokenDecimals {
        denom: String,
        decimals: u8,
    },
    MigratePair {
        contract: String,
        code_id: Option<u64>,
    },
}

#[cw_serde]
pub enum MigrateMsg {}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Pair {
        asset_infos: [AssetInfo; 2],
    },
    Pairs {
        start_after: Option<[AssetInfo; 2]>,
        limit: Option<u32>,
    },
    // NativeTokenDecimals {
    //     denom: String,
    // },
}
