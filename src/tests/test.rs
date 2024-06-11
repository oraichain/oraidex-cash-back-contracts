use std::str::FromStr;

use cosmwasm_std::{Addr, Decimal, Uint128};
use oraiswap::asset::AssetInfo;

use crate::{
    msg::{ExecuteMsg, QueryMsg},
    state::{Campaign, Config},
};

use super::hepler::MockApp;

#[test]
fn test_instantiate() {
    let mut mock_app = MockApp::new(&[("admin", &[])]);
    let owner = "admin";
    let token = "oraix";

    let oraix_token = mock_app.create_token(owner, token, 0u128);
    let underlying_token = AssetInfo::Token {
        contract_addr: oraix_token,
    };
    let cash_back_addr = mock_app
        .create_cash_back_contract(owner, underlying_token.clone(), None)
        .unwrap();

    // query config
    let config: Config = mock_app
        .query(cash_back_addr, &QueryMsg::Config {})
        .unwrap();

    assert_eq!(
        config,
        Config {
            owner: Addr::unchecked(owner),
            underlying_token,
            rules: vec![]
        }
    )
}

#[test]
fn test_update_config() {
    let mut mock_app = MockApp::new(&[("admin", &[])]);
    let owner = "admin";
    let token = "oraix";

    let oraix_token = mock_app.create_token(owner, token, 0u128);
    let underlying_token = AssetInfo::Token {
        contract_addr: oraix_token,
    };
    let cash_back_addr = mock_app
        .create_cash_back_contract(owner, underlying_token.clone(), None)
        .unwrap();

    let new_token = AssetInfo::NativeToken {
        denom: "orai".to_string(),
    };
    let mut new_rules = vec![
        (Uint128::from(100u128), Decimal::from_str("0.1").unwrap()),
        (Uint128::from(200u128), Decimal::from_str("0.2").unwrap()),
        (Uint128::from(400u128), Decimal::from_str("0.4").unwrap()),
        (Uint128::from(300u128), Decimal::from_str("0.3").unwrap()),
    ];
    let new_owner = Addr::unchecked("new_owner");
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some(new_owner.clone()),
        underlying_token: Some(new_token.clone()),
        rules: Some(new_rules.clone()),
    };

    // update failed, unauthorized
    let err = mock_app.execute(new_owner.clone(), cash_back_addr.clone(), &msg, &[]);
    assert!(err.is_err());

    // update successful
    mock_app
        .execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[])
        .unwrap();
    // query config
    let config: Config = mock_app
        .query(cash_back_addr, &QueryMsg::Config {})
        .unwrap();

    new_rules.sort_by(|a, b| b.0.cmp(&a.0));
    assert_eq!(
        config,
        Config {
            owner: new_owner,
            underlying_token: new_token,
            rules: new_rules,
        }
    )
}

#[test]
fn test_create_campaign() {
    let mut mock_app = MockApp::new(&[("admin", &[])]);
    let owner = "admin";
    let token = "oraix";

    let oraix_token = mock_app.create_token(owner, token, 0u128);
    let underlying_token = AssetInfo::Token {
        contract_addr: oraix_token,
    };
    let cash_back_addr = mock_app
        .create_cash_back_contract(owner, underlying_token.clone(), None)
        .unwrap();

    let current = mock_app.app.block_info().time.seconds();

    let msg = ExecuteMsg::CreateCampaign {
        start: current,
        end: current + 100,
        reward_token: underlying_token.clone(),
        total_reward: Uint128::from(1000000u128),
    };
    // create failed, unauthorized
    let err = mock_app.execute(Addr::unchecked("sender"), cash_back_addr.clone(), &msg, &[]);
    assert!(err.is_err());

    // create failed, time range invalid
    let msg = ExecuteMsg::CreateCampaign {
        start: current + 200,
        end: current + 100,
        reward_token: underlying_token.clone(),
        total_reward: Uint128::from(1000000u128),
    };
    let err = mock_app.execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[]);
    assert!(err.is_err());

    // create successful
    let msg = ExecuteMsg::CreateCampaign {
        start: current + 0,
        end: current + 100,
        reward_token: underlying_token.clone(),
        total_reward: Uint128::from(1000000u128),
    };
    mock_app
        .execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[])
        .unwrap();

    let last_round_id: u64 = mock_app
        .query(cash_back_addr.clone(), &QueryMsg::LastCampaignId {})
        .unwrap();
    assert_eq!(last_round_id, 1);
    let last_round: Campaign = mock_app
        .query(cash_back_addr.clone(), &QueryMsg::LastCampaign {})
        .unwrap();
    assert_eq!(
        last_round,
        Campaign {
            id: 1,
            start: current + 0,
            end: current + 100,
            reward_token: underlying_token.clone(),
            total_reward: Uint128::from(1000000u128),
            distributed_amount: Uint128::zero()
        }
    );

    // create new round failed, last round not finished
    let msg = ExecuteMsg::CreateCampaign {
        start: current + 200,
        end: current + 300,
        reward_token: underlying_token.clone(),
        total_reward: Uint128::from(1000000u128),
    };
    let err = mock_app.execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[]);
    assert!(err.is_err());

    // increase time
    mock_app.app.update_block(|block| {
        block.time = block.time.plus_seconds(300);
        block.height += 1;
    });

    // create new round success
    let msg = ExecuteMsg::CreateCampaign {
        start: current + 200,
        end: current + 300,
        reward_token: underlying_token.clone(),
        total_reward: Uint128::from(1000000u128),
    };
    mock_app
        .execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[])
        .unwrap();
}
#[test]
fn test_trigger_cash_back() {
    let mut mock_app = MockApp::new(&[("admin", &[])]);
    let owner = "admin";
    let token = "oraix";

    let oraix_token = mock_app.create_token(owner, token, 0u128);
    let underlying_token = AssetInfo::Token {
        contract_addr: oraix_token,
    };
    let cash_back_addr = mock_app
        .create_cash_back_contract(owner, underlying_token.clone(), None)
        .unwrap();

    // let msg =
    // trigger failed because this contract didn't register yet
}
