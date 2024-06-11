use cosmwasm_std::{
    coin, to_json_binary, Addr, BankMsg, CosmosMsg, Decimal, QuerierWrapper, StdError, StdResult,
    Uint128, WasmMsg,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use oraiswap::asset::AssetInfo;

pub fn build_transfer_msg(
    token: &AssetInfo,
    amount: Uint128,
    receiver: &Addr,
) -> StdResult<CosmosMsg> {
    match token {
        AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: receiver.to_string(),
                amount,
            })?,
            funds: vec![],
        })),
        AssetInfo::NativeToken { denom } => {
            let send_amount = coin(amount.u128(), denom);

            Ok(CosmosMsg::Bank(BankMsg::Send {
                to_address: receiver.to_string(),
                amount: vec![send_amount],
            }))
        }
    }
}

pub fn query_asset_balance(querier: &QuerierWrapper, user: &Addr, token: &AssetInfo) -> Uint128 {
    match token {
        AssetInfo::NativeToken { denom } => {
            querier
                .query_balance(user.to_string(), denom)
                .unwrap_or_default()
                .amount
        }
        AssetInfo::Token { contract_addr } => {
            querier
                .query_wasm_smart(
                    contract_addr.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: user.to_string(),
                    },
                )
                .unwrap_or(BalanceResponse {
                    balance: Uint128::zero(),
                })
                .balance
        }
    }
}

pub fn validate_cash_back_rule(rules: &[(Uint128, Decimal)]) -> StdResult<()> {
    if rules
        .iter()
        .any(|&(_, percent)| percent.gt(&Decimal::one()))
    {
        return Err(StdError::generic_err("Cash back percent must be lte 1"));
    }
    Ok(())
}
