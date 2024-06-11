use std::collections::HashMap;

use cosmwasm_std::{Addr, Coin, Decimal, Empty, QuerierWrapper, StdResult, Uint128};
use cw_multi_test::{next_block, App, AppResponse, Contract, Executor};
use oraiswap::asset::AssetInfo;
use serde::{de::DeserializeOwned, Serialize};

use crate::msg;

#[macro_export]
macro_rules! create_entry_points_testing {
    ($contract:ident) => {
        cw_multi_test::ContractWrapper::new(
            $contract::contract::execute,
            $contract::contract::instantiate,
            $contract::contract::query,
        )
    };
}

pub struct MockApp {
    pub app: App,
    token_map: HashMap<String, Addr>, // map token name to address
    pub token_id: u64,
}

impl MockApp {
    pub fn new(init_balances: &[(&str, &[Coin])]) -> Self {
        let mut app = App::new(|router, _, storage| {
            for (owner, init_funds) in init_balances.iter() {
                router
                    .bank
                    .init_balance(
                        storage,
                        &Addr::unchecked(owner.to_owned()),
                        init_funds.to_vec(),
                    )
                    .unwrap();
            }
        });

        // default token is cw20_base
        let token_id = app.store_code(Box::new(crate::create_entry_points_testing!(cw20_base)));

        MockApp {
            app,
            token_id,
            token_map: HashMap::new(),
        }
    }

    pub fn set_token_contract(&mut self, code: Box<dyn Contract<Empty>>) {
        self.token_id = self.upload(code);
    }

    pub fn upload(&mut self, code: Box<dyn Contract<Empty>>) -> u64 {
        let code_id = self.app.store_code(code);
        self.app.update_block(next_block);
        code_id
    }

    pub fn instantiate<T: Serialize>(
        &mut self,
        code_id: u64,
        sender: Addr,
        init_msg: &T,
        send_funds: &[Coin],
        label: &str,
    ) -> Result<Addr, String> {
        let contract_addr = self
            .app
            .instantiate_contract(code_id, sender, init_msg, send_funds, label, None)
            .map_err(|err| err.to_string())?;
        self.app.update_block(next_block);
        Ok(contract_addr)
    }

    pub fn execute<T: Serialize + std::fmt::Debug>(
        &mut self,
        sender: Addr,
        contract_addr: Addr,
        msg: &T,
        send_funds: &[Coin],
    ) -> Result<AppResponse, String> {
        let response = self
            .app
            .execute_contract(sender, contract_addr, msg, send_funds)
            .map_err(|err| err.to_string())?;

        self.app.update_block(next_block);

        Ok(response)
    }

    pub fn query<T: DeserializeOwned, U: Serialize>(
        &self,
        contract_addr: Addr,
        msg: &U,
    ) -> StdResult<T> {
        self.app.wrap().query_wasm_smart(contract_addr, msg)
    }

    pub fn as_querier(&self) -> QuerierWrapper {
        self.app.wrap()
    }

    pub fn create_token(&mut self, owner: &str, token: &str, initial_amount: u128) -> Addr {
        let addr = self
            .instantiate(
                self.token_id,
                Addr::unchecked(owner),
                &cw20_base::msg::InstantiateMsg {
                    name: token.to_string(),
                    symbol: token.to_string(),
                    decimals: 6,
                    initial_balances: vec![cw20::Cw20Coin {
                        address: owner.to_string(),
                        amount: initial_amount.into(),
                    }],
                    mint: Some(cw20::MinterResponse {
                        minter: owner.to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                &[],
                "cw20",
            )
            .unwrap();
        self.token_map.insert(token.to_string(), addr.clone());
        addr
    }

    /// external method
    pub fn create_cash_back_contract(
        &mut self,
        owner: &str,
        underlying_token: AssetInfo,
        rules: Option<Vec<(Uint128, Decimal)>>,
    ) -> Result<Addr, String> {
        let code_id = self.upload(Box::new(create_entry_points_testing!(crate)));
        self.instantiate(
            code_id,
            Addr::unchecked(owner),
            &msg::InstantiateMsg {
                owner: Addr::unchecked(owner),
                underlying_token,
                rules,
            },
            &[],
            "cash-back-contract",
        )
    }
}
