use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};
use oraiswap::asset::{Asset, AssetInfo};

use crate::state::{Campaign, Config};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub underlying_token: AssetInfo,
    pub rules: Option<Vec<(Uint128, Decimal)>>, // contain list conditions: balance - % cash back
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<Addr>,
        underlying_token: Option<AssetInfo>,
        rules: Option<Vec<(Uint128, Decimal)>>,
    },
    // Allow only whitelisted contracts to trigger cash back
    WhitelistContract {
        contract: Addr,
    },
    // Exclude contracts that are eligible for cash back
    RemoveContract {
        contract: Addr,
    },
    // create cash back campaign
    CreateCampaign {
        start: u64,
        end: u64,
        reward_token: AssetInfo,
        total_reward: Uint128,
    },
    // edit campaign
    EditCampaign {
        id: u64,
        start: Option<u64>,
        end: Option<u64>,
        total_reward: Option<Uint128>,
    },
    // called by a whitelisted contract, this function triggers a cashback for the user
    TriggerCashBack {
        user: Addr,
        tokens: Vec<Asset>,
    },
    // TODO: Move to Sudo entrypoint
    CashBack {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(Campaign)]
    Campaign { id: u64 },
    #[returns(Campaign)]
    LastCampaign {},
    #[returns(u64)]
    LastCampaignId {},
    #[returns(Vec<String>)]
    WhitelistContract {},
    #[returns(Uint128)]
    PendingCashBack { user: Addr },
}
