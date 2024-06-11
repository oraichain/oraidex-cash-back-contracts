use std::str::FromStr;

use cosmwasm_std::{Addr, Decimal, Uint128};
use oraiswap::asset::{Asset, AssetInfo};

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
    let current = mock_app.app.block_info().time.seconds();
    let msg = ExecuteMsg::CreateCampaign {
        start: current,
        end: current + 300,
        reward_token: underlying_token.clone(),
        total_reward: Uint128::from(1000000u128),
    };
    mock_app
        .execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[])
        .unwrap();

    // try update campaign failed, unauthorized
    let msg = ExecuteMsg::EditCampaign {
        id: 2,
        start: Some(current),
        end: Some(current + 200),
        total_reward: Some(Uint128::from(2000000u128)),
    };
    let err = mock_app.execute(Addr::unchecked("sender"), cash_back_addr.clone(), &msg, &[]);
    assert!(err.is_err());

    // update successful
    mock_app
        .execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[])
        .unwrap();
    let last_round: Campaign = mock_app
        .query(cash_back_addr.clone(), &QueryMsg::LastCampaign {})
        .unwrap();
    assert_eq!(
        last_round,
        Campaign {
            id: 2,
            start: current,
            end: current + 200,
            reward_token: underlying_token.clone(),
            total_reward: Uint128::from(2000000u128),
            distributed_amount: Uint128::zero()
        }
    );

    // after finish, can not update
    mock_app.app.update_block(|block| {
        block.time = block.time.plus_seconds(300);
        block.height += 1;
    });
    let err = mock_app.execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[]);
    assert!(err.is_err())
}
#[test]
fn test_whitelist_contract() {
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

    let msg = ExecuteMsg::WhitelistContract {
        contract: Addr::unchecked("contract001"),
    };

    // register failed, unauthorized
    let err = mock_app.execute(Addr::unchecked("sender"), cash_back_addr.clone(), &msg, &[]);
    assert!(err.is_err());

    // register successful
    mock_app
        .execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[])
        .unwrap();

    // register other contract
    let msg = ExecuteMsg::WhitelistContract {
        contract: Addr::unchecked("contract002"),
    };
    mock_app
        .execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[])
        .unwrap();

    let whitelist_contract: Vec<String> = mock_app
        .query(cash_back_addr.clone(), &QueryMsg::WhitelistContract {})
        .unwrap();
    assert_eq!(whitelist_contract, vec!["contract001", "contract002"]);

    // try remove failed, unauthorized
    let msg = ExecuteMsg::RemoveContract {
        contract: Addr::unchecked("contract001"),
    };
    let err = mock_app.execute(Addr::unchecked("sender"), cash_back_addr.clone(), &msg, &[]);
    assert!(err.is_err());
    // remove successful
    mock_app
        .execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[])
        .unwrap();
    // remove failed, contract not registered yet
    let msg = ExecuteMsg::RemoveContract {
        contract: Addr::unchecked("contract001"),
    };
    let err = mock_app.execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[]);
    assert!(err.is_err());

    let whitelist_contract: Vec<String> = mock_app
        .query(cash_back_addr.clone(), &QueryMsg::WhitelistContract {})
        .unwrap();
    assert_eq!(whitelist_contract, vec!["contract002"]);
}

#[test]
fn test_trigger_cash_back() {
    let mut mock_app = MockApp::new(&[("admin", &[])]);
    let owner = "admin";
    let token = "oraix";
    let rules = vec![
        (Uint128::from(100u128), Decimal::from_str("0.1").unwrap()),
        (Uint128::from(200u128), Decimal::from_str("0.2").unwrap()),
        (Uint128::from(400u128), Decimal::from_str("0.4").unwrap()),
        (Uint128::from(300u128), Decimal::from_str("0.3").unwrap()),
    ];

    let oraix_token = mock_app.create_token(owner, token, 0u128);
    let underlying_token = AssetInfo::Token {
        contract_addr: oraix_token.clone(),
    };
    let cash_back_addr = mock_app
        .create_cash_back_contract(owner, underlying_token.clone(), Some(rules))
        .unwrap();

    mock_app
        .mint_token(
            owner,
            cash_back_addr.as_str(),
            oraix_token.as_str(),
            1000000u128,
        )
        .unwrap();

    let msg = ExecuteMsg::WhitelistContract {
        contract: Addr::unchecked("contract001"),
    };
    mock_app
        .execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[])
        .unwrap();

    let orai = AssetInfo::NativeToken {
        denom: "orai".to_string(),
    };
    let usdt = mock_app.create_token(owner, token, 0u128);
    let usdt_info = AssetInfo::Token {
        contract_addr: usdt,
    };

    let msg = ExecuteMsg::TriggerCashBack {
        user: Addr::unchecked("addr000"),
        tokens: vec![
            Asset {
                info: orai.clone(),
                amount: Uint128::from(1000u128),
            },
            Asset {
                info: usdt_info.clone(),
                amount: Uint128::from(2000u128),
            },
        ],
    };

    // case 1:  contract not register yet
    mock_app
        .execute(
            Addr::unchecked("contract002"),
            cash_back_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
    let pending_cash_back: Uint128 = mock_app
        .query(
            cash_back_addr.clone(),
            &QueryMsg::PendingCashBack {
                user: Addr::unchecked("addr000"),
            },
        )
        .unwrap();
    assert_eq!(pending_cash_back, Uint128::zero());

    // case 2: don;t exist campaign
    mock_app
        .execute(
            Addr::unchecked("contract001"),
            cash_back_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
    let pending_cash_back: Uint128 = mock_app
        .query(
            cash_back_addr.clone(),
            &QueryMsg::PendingCashBack {
                user: Addr::unchecked("addr000"),
            },
        )
        .unwrap();
    assert_eq!(pending_cash_back, Uint128::zero());

    // case 3: trigger success
    let current = mock_app.app.block_info().time.seconds();
    let msg = ExecuteMsg::CreateCampaign {
        start: current,
        end: current + 300,
        reward_token: underlying_token.clone(),
        total_reward: Uint128::from(1000u128),
    };
    mock_app
        .execute(Addr::unchecked(owner), cash_back_addr.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::TriggerCashBack {
        user: Addr::unchecked("addr000"),
        tokens: vec![
            Asset {
                info: orai.clone(),
                amount: Uint128::from(1000u128),
            },
            Asset {
                info: usdt_info.clone(),
                amount: Uint128::from(2000u128),
            },
        ],
    };

    mock_app
        .execute(
            Addr::unchecked("contract001"),
            cash_back_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // pending still 0, because balance  = 0 => no refund
    let pending_cash_back: Uint128 = mock_app
        .query(
            cash_back_addr.clone(),
            &QueryMsg::PendingCashBack {
                user: Addr::unchecked("addr000"),
            },
        )
        .unwrap();
    assert_eq!(pending_cash_back, Uint128::zero());

    // update balance
    mock_app
        .mint_token(owner, "addr000", oraix_token.as_str(), 150u128)
        .unwrap();

    // try trigger cash back again, fee is tier 1 (10%) => cashBackAmount = 100 + 200 = 300
    mock_app
        .execute(
            Addr::unchecked("contract001"),
            cash_back_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
    let pending_cash_back: Uint128 = mock_app
        .query(
            cash_back_addr.clone(),
            &QueryMsg::PendingCashBack {
                user: Addr::unchecked("addr000"),
            },
        )
        .unwrap();
    assert_eq!(pending_cash_back, Uint128::from(300u128));

    // update balance to tier 2: 20%
    mock_app
        .mint_token(owner, "addr000", oraix_token.as_str(), 100u128)
        .unwrap();
    mock_app
        .execute(
            Addr::unchecked("contract001"),
            cash_back_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
    let pending_cash_back: Uint128 = mock_app
        .query(
            cash_back_addr.clone(),
            &QueryMsg::PendingCashBack {
                user: Addr::unchecked("addr000"),
            },
        )
        .unwrap();
    assert_eq!(pending_cash_back, Uint128::from(900u128));

    // try calc cash back
    let cash_back_msg = ExecuteMsg::CashBack {};
    mock_app
        .execute(
            Addr::unchecked(owner),
            cash_back_addr.clone(),
            &cash_back_msg,
            &[],
        )
        .unwrap();

    // after call this func, pending become zero
    let pending_cash_back: Uint128 = mock_app
        .query(
            cash_back_addr.clone(),
            &QueryMsg::PendingCashBack {
                user: Addr::unchecked("addr000"),
            },
        )
        .unwrap();
    assert_eq!(pending_cash_back, Uint128::zero());

    // continue trigger cashback adn  reach limit of campaign (remaining 100 token)
    mock_app
        .execute(
            Addr::unchecked("contract001"),
            cash_back_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
    let pending_cash_back: Uint128 = mock_app
        .query(
            cash_back_addr.clone(),
            &QueryMsg::PendingCashBack {
                user: Addr::unchecked("addr000"),
            },
        )
        .unwrap();
    assert_eq!(pending_cash_back, Uint128::from(100u128));

    // query last campaign
    let last_round: Campaign = mock_app
        .query(cash_back_addr.clone(), &QueryMsg::LastCampaign {})
        .unwrap();
    assert_eq!(
        last_round,
        Campaign {
            id: 1,
            start: current,
            end: current + 300,
            reward_token: underlying_token.clone(),
            total_reward: Uint128::from(1000u128),
            distributed_amount: Uint128::from(1000u128),
        }
    );
}
