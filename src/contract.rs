#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use oraiswap::asset::AssetInfo;
// use cw2::set_contract_version;

use crate::cash_back::{execute_cash_back, execute_trigger_cash_back};
use crate::error::ContractError;
use crate::helpers::validate_cash_back_rule;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Campaign, Config, CAMPAIGN, CONFIG, LAST_CAMPAIGN_ID, WHITELIST_CONTRACT};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cash-back-contracts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut rules = msg.rules.unwrap_or_default();
    validate_cash_back_rule(&rules)?;
    rules.sort_by(|a, b| b.0.cmp(&a.0));

    CONFIG.save(
        deps.storage,
        &Config {
            owner: msg.owner,
            underlying_token: msg.underlying_token,
            rules,
        },
    )?;
    LAST_CAMPAIGN_ID.save(deps.storage, &0)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            underlying_token,
            rules,
        } => execute_update_config(deps, info, owner, underlying_token, rules),
        ExecuteMsg::WhitelistContract { contract } => {
            execute_whitelist_contract(deps, info, contract)
        }
        ExecuteMsg::RemoveContract { contract } => execute_remove_contract(deps, info, contract),
        ExecuteMsg::CreateCampaign {
            start,
            end,
            reward_token,
            total_reward,
        } => execute_create_campaign(deps, env, info, start, end, reward_token, total_reward),
        ExecuteMsg::EditCampaign {
            id,
            start,
            end,
            total_reward,
        } => execute_edit_campaign(deps, env, info, id, start, end, total_reward),
        ExecuteMsg::TriggerCashBack { user, tokens } => {
            execute_trigger_cash_back(deps, env, info, user, tokens)
        }
        ExecuteMsg::CashBack {} => execute_cash_back(deps),
    }
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<Addr>,
    underlying_token: Option<AssetInfo>,
    rules: Option<Vec<(Uint128, Decimal)>>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = owner;
    }

    if let Some(underlying_token) = underlying_token {
        config.underlying_token = underlying_token;
    }

    if let Some(mut rules) = rules {
        validate_cash_back_rule(&rules)?;
        rules.sort_by(|a, b| b.0.cmp(&a.0));
        config.rules = rules;
    }
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default().add_attribute("action", "update_config"))
}
fn execute_whitelist_contract(
    deps: DepsMut,
    info: MessageInfo,
    contract: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    WHITELIST_CONTRACT.add_hook(deps.storage, contract.clone())?;

    Ok(Response::new().add_attributes(vec![
        ("action", "whitelist_contract"),
        ("contract", contract.as_str()),
    ]))
}

fn execute_remove_contract(
    deps: DepsMut,
    info: MessageInfo,
    contract: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    WHITELIST_CONTRACT.remove_hook(deps.storage, contract.clone())?;

    Ok(Response::new().add_attributes(vec![
        ("action", "remove_contract"),
        ("contract", contract.as_str()),
    ]))
}

fn execute_create_campaign(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start: u64,
    end: u64,
    reward_token: AssetInfo,
    total_reward: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if start > end {
        return Err(ContractError::InvalidCampaignTime {});
    }
    let last_id = LAST_CAMPAIGN_ID.may_load(deps.storage)?.unwrap_or_default();
    if last_id > 0 {
        let last_campaign = CAMPAIGN.load(deps.storage, last_id)?;
        if !last_campaign.is_finished(&env) {
            return Err(ContractError::Std(StdError::generic_err(
                "Last campaign is not over yet",
            )));
        }
    }
    let id = last_id + 1;

    CAMPAIGN.save(
        deps.storage,
        id,
        &Campaign {
            id,
            start,
            end,
            total_reward,
            reward_token: reward_token.clone(),
            distributed_amount: Uint128::zero(),
        },
    )?;
    LAST_CAMPAIGN_ID.save(deps.storage, &last_id)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "create_campaign"),
        ("campaign_id", &id.to_string()),
        ("start", &start.to_string()),
        ("end", &end.to_string()),
        ("reward_token", &format!("{:?}", reward_token)),
        ("total_reward", &total_reward.to_string()),
    ]))
}

fn execute_edit_campaign(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
    start: Option<u64>,
    end: Option<u64>,
    total_reward: Option<Uint128>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut campaign = CAMPAIGN.load(deps.storage, id)?;
    if campaign.is_finished(&env) {
        return Err(ContractError::CampaignEnded {});
    }

    if let Some(start) = start {
        campaign.start = start;
    }
    if let Some(end) = end {
        campaign.end = end;
    }
    if let Some(total_reward) = total_reward {
        campaign.total_reward = total_reward;
    }
    if campaign.start > campaign.end {
        return Err(ContractError::InvalidCampaignTime {});
    }

    Ok(Response::new().add_attributes(vec![
        ("action", "edit_campaign"),
        ("campaign_id", &id.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::Campaign { id } => to_json_binary(&CAMPAIGN.load(deps.storage, id)?),
        QueryMsg::LastCampaign {} => to_json_binary(&query_last_campaign(deps)?),
        QueryMsg::LastCampaignId {} => to_json_binary(&LAST_CAMPAIGN_ID.load(deps.storage)?),
    }
}

fn query_last_campaign(deps: Deps) -> StdResult<Campaign> {
    let last_id = LAST_CAMPAIGN_ID.load(deps.storage)?;
    CAMPAIGN.load(deps.storage, last_id)
}

#[cfg(test)]
mod tests {}
