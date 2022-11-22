use cosmwasm_schema::write_api;

use halo_pair::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        migrate: MigrateMsg,
        execute: ExecuteMsg,
        // reply:,
        // query: QueryMsg,
    }
}
