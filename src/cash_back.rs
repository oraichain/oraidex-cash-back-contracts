use cosmwasm_std::{
    Addr, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use oraiswap::asset::{Asset, AssetInfo};

use crate::{
    helpers::{build_transfer_msg, query_asset_balance},
    state::{
        read_all_pending_cash_back, CAMPAIGN, CONFIG, LAST_CAMPAIGN_ID, PENDING_CASH_BACK,
        USER_CASH_BACK, WHITELIST_CONTRACT,
    },
    ContractError,
};

pub fn execute_cash_back(deps: DepsMut) -> Result<Response, ContractError> {
    let last_id = LAST_CAMPAIGN_ID.may_load(deps.storage)?.unwrap_or_default();
    if last_id == 0 {
        return Ok(Response::default());
    }
    let campaign = CAMPAIGN.load(deps.storage, last_id)?;
    let pending = read_all_pending_cash_back(deps.storage);

    let msgs: Vec<CosmosMsg> = pending
        .iter()
        .map(|item| build_transfer_msg(&campaign.reward_token, item.1, &item.0).unwrap())
        .collect();

    // remove all pending cash back
    PENDING_CASH_BACK.clear(deps.storage);

    Ok(Response::new()
        .add_attribute("action", "execute_cash_back")
        .add_messages(msgs))
}

pub fn execute_trigger_cash_back(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user: Addr,
    tokens: Vec<Asset>,
) -> Result<Response, ContractError> {
    // check sender must be whitelisted
    if !WHITELIST_CONTRACT
        .query_hook(deps.as_ref(), info.sender.to_string())
        .unwrap_or_default()
    {
        return Ok(Response::default());
    }

    let last_id = LAST_CAMPAIGN_ID.may_load(deps.storage)?.unwrap_or_default();
    if last_id == 0 {
        return Ok(Response::default());
    }
    let mut campaign = CAMPAIGN.load(deps.storage, last_id)?;
    if !campaign.in_progress(&env) || campaign.distributed_amount == campaign.total_reward {
        return Ok(Response::default());
    }

    let cash_back_percent = calc_cash_back_percent(deps.as_ref(), &user)?;

    if cash_back_percent.is_zero() {
        return Ok(Response::default());
    }

    let cash_back_tokens: Vec<Asset> = tokens
        .iter()
        .map(|token| Asset {
            info: token.info.clone(),
            amount: token.amount * cash_back_percent,
        })
        .collect();

    // convert fee tokens to cashBackToken
    let cash_back_amount =
        convert_to_reward_token(deps.as_ref(), &cash_back_tokens, &campaign.reward_token)?
            .min(campaign.total_reward - campaign.distributed_amount);

    PENDING_CASH_BACK.update(deps.storage, &user, |pending| -> StdResult<_> {
        let mut pending = pending.unwrap_or_default();
        pending += cash_back_amount;
        Ok(pending)
    })?;
    USER_CASH_BACK.update(deps.storage, (last_id, &user), |total| -> StdResult<_> {
        let mut total = total.unwrap_or_default();
        total += cash_back_amount;
        Ok(total)
    })?;

    campaign.distributed_amount += cash_back_amount;

    CAMPAIGN.save(deps.storage, last_id, &campaign)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "trigger_cash_back"),
        ("user", user.as_str()),
        ("cash_back_amount", &cash_back_amount.to_string()),
    ]))
}

pub fn calc_cash_back_percent(deps: Deps, user: &Addr) -> StdResult<Decimal> {
    let config = CONFIG.load(deps.storage)?;

    let balance = query_asset_balance(&deps.querier, user, &config.underlying_token);

    // Because amount sort by desc, so find best level by iterating through rules and finding the first matching rule
    let percent = config
        .rules
        .iter()
        .find_map(|(threshold, cash_back_percent)| {
            if *threshold <= balance {
                Some(*cash_back_percent)
            } else {
                None
            }
        })
        .unwrap_or(Decimal::zero());

    Ok(percent)
}

pub fn convert_to_reward_token(
    deps: Deps,
    tokens: &Vec<Asset>,
    reward_token: &AssetInfo,
) -> StdResult<Uint128> {
    let mut total_cash_back = Uint128::zero();
    let reward_token_price = get_token_price(deps, reward_token)?;
    for token in tokens {
        let price = get_token_price(deps, &token.info)?;
        total_cash_back += token.amount * (price.checked_div(reward_token_price).unwrap())
    }

    Ok(total_cash_back)
}

// TODO: calc token price
pub fn get_token_price(_deps: Deps, _token: &AssetInfo) -> StdResult<Decimal> {
    Ok(Decimal::one())
}
