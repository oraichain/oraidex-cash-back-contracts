use cosmwasm_std::Addr;
use oraiswap::asset::AssetInfo;

use crate::{msg::QueryMsg, state::Config};

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
