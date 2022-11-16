
use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;
use cosmwasm_std::{Addr, Api, CanonicalAddr, QuerierWrapper, StdResult};

use crate::asset::{Asset, AssetInfo, AssetInfoRaw};

pub const PAIR_INFO: Item<PairInfoRaw> = Item::new("pair_info");
pub const COMMISSION_RATE: &str = "0.003";

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct PairsResponse {
    pub pairs: Vec<PairInfo>,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct PairInfo {
    pub asset_infos: [AssetInfo; 2],
    pub contract_addr: String,
    pub liquidity_token: String,
    pub asset_decimals: [u8; 2],
}

#[cw_serde]
pub struct PairInfoRaw {
    pub asset_infos: [AssetInfoRaw; 2],
    pub contract_addr: CanonicalAddr,
    pub liquidity_token: CanonicalAddr,
    pub asset_decimals: [u8; 2],
}

impl PairInfoRaw {
    pub fn to_normal(&self, api: &dyn Api) -> StdResult<PairInfo> {
        Ok(PairInfo {
            liquidity_token: api.addr_humanize(&self.liquidity_token)?.to_string(),
            contract_addr: api.addr_humanize(&self.contract_addr)?.to_string(),
            asset_infos: [
                self.asset_infos[0].to_normal(api)?,
                self.asset_infos[1].to_normal(api)?,
            ],
            asset_decimals: self.asset_decimals,
        })
    }

    // query information of a pair provided by the contract_addr - address of pair contract
    // @contract_addr: address of pair contract
    // @return: PairInfo - information of the pair (type and amount of each asset)
    pub fn query_pools(
        &self,
        querier: &QuerierWrapper,
        api: &dyn Api,
        contract_addr: Addr,
    ) -> StdResult<[Asset; 2]> {
        // query the first asset information
        let info_0: AssetInfo = self.asset_infos[0].to_normal(api)?;

        // query the second asset information
        let info_1: AssetInfo = self.asset_infos[1].to_normal(api)?;
        Ok([
            // query the first asset information in the pool provided by the contract_addr
            Asset {
                amount: info_0.query_pool(querier, api, contract_addr.clone())?,
                info: info_0,
            },
            // query the second asset information in the pool provided by the contract_addr
            Asset {
                amount: info_1.query_pool(querier, api, contract_addr)?,
                info: info_1,
            },
        ])
    }
}
