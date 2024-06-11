use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Env, Order, Storage, Uint128};
use cw_controllers::Hooks;
use cw_storage_plus::{Item, Map};
use oraiswap::asset::AssetInfo;

pub const WHITELIST_CONTRACT: Hooks = Hooks::new("whitelist_contract");
pub const CONFIG: Item<Config> = Item::new("config");
// campaign detail
pub const CAMPAIGN: Map<u64, Campaign> = Map::new("campaign");
// last campaign id
pub const LAST_CAMPAIGN_ID: Item<u64> = Item::new("last_campaign_id");
// store pending cash back amount
pub const PENDING_CASH_BACK: Map<&Addr, Uint128> = Map::new("pending_cash_back");
// mapping from (campaignId, user) -> total amount cash back in this campaign
pub const USER_CASH_BACK: Map<(u64, &Addr), Uint128> = Map::new("user_cash_back");

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub underlying_token: AssetInfo,
    pub rules: Vec<(Uint128, Decimal)>, // contain list conditions: balance - % cash back
}

#[cw_serde]
pub struct Campaign {
    pub id: u64,
    pub start: u64,
    pub end: u64,
    pub total_reward: Uint128,
    pub reward_token: AssetInfo,
    pub distributed_amount: Uint128,
}

impl Campaign {
    pub fn is_finished(&self, env: &Env) -> bool {
        self.end < env.block.time.seconds()
    }

    pub fn in_progress(&self, env: &Env) -> bool {
        let current = env.block.time.seconds();
        self.start <= current && self.end >= current
    }
}
pub fn read_all_pending_cash_back(storage: &dyn Storage) -> Vec<(Addr, Uint128)> {
    PENDING_CASH_BACK
        .range(storage, None, None, Order::Ascending)
        .map(|item| item.unwrap())
        .collect()
}
