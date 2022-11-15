use cw_storage_plus::{Bound, Item, Map};

pub const ALLOW_NATIVE_TOKENS: Map<&[u8], u8> = Map::new("allow_native_token");