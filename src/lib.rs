extern crate core;

mod consts;
mod proposals;
mod types;
mod views;

use crate::types::{Action, Proposal, ProposalOutput, ProposalStatus, VersionedProposal};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, TreeMap},
    env, ext_contract, near_bindgen, serde_json,
    serde_json::Value,
    AccountId, Balance, BorshStorageKey, PanicOnDefault, PromiseResult,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, convert::TryFrom};

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn callback_promise_result() -> bool;
    fn callback_add_proposal_result() -> u64;
    fn on_proposal_callback(&mut self, proposal_id: u64) -> PromiseOrValue<()>;
    fn proposal_result_callback(&mut self) -> ProposalOutput;
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Policy,
    Delegations,
    Proposals,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    /// Amount of $NEAR locked for bonds.
    pub locked_amount: Balance,
    /// Vote staking contract id. That contract must have this account as owner.
    pub staking_id: Option<AccountId>,
    /// Delegated  token total amount.
    pub total_delegation_amount: Balance,
    /// Delegations per user.
    pub delegations: LookupMap<AccountId, Balance>,
    /// Last available id for the proposals.
    pub last_proposal_id: u64,
    /// Proposal map from ID to proposal information.
    pub proposals: TreeMap<u64, VersionedProposal>,
    /// List of registries
    registries: HashMap<AccountId, Vec<RegistryData>>,
    /// AstroDao contract
    dao: AccountId,
    /// Voting Power
    voting_power: HashMap<u64, Vec<(AccountId, Balance, Action)>>,
}

#[near_bindgen]
#[derive(
    BorshDeserialize, BorshSerialize, PanicOnDefault, Serialize, Deserialize, Clone, Debug,
)]
pub struct RegistryData {
    dao: String,
    name: String,
    owner: AccountId,
    unique_identifier: AccountId,
    #[serde(skip_serializing)]
    row: Vec<Row>,
    #[serde(skip_serializing)]
    column: Vec<Column>,
}

#[near_bindgen]
#[derive(
    BorshDeserialize, BorshSerialize, PanicOnDefault, Serialize, Deserialize, Clone, Debug,
)]
pub struct Row {
    unique_identifier: u64,
    #[serde(skip_serializing)]
    data: Vec<u8>,
}

#[near_bindgen]
#[derive(
    BorshDeserialize, BorshSerialize, PanicOnDefault, Serialize, Deserialize, Clone, Debug,
)]
pub struct Column {
    unique_identifier: u64,
    #[serde(skip_serializing)]
    data: Vec<u8>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
pub enum Status {
    Open,
    InVoting,
    Change,
}

#[near_bindgen]
impl RegistryData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        row_data: Vec<Value>,
        column_data: Vec<Value>,
        owner: AccountId,
        dao: String,
        columns_identifiers: Vec<u64>,
        rows_identifiers: Vec<u64>,
    ) -> Self {
        let mut row: Vec<Row> = Vec::new();
        for row_bytes in row_data {
            for identifier in &rows_identifiers {
                row.push(Row {
                    unique_identifier: *identifier,
                    data: row_bytes.to_string().into_bytes(),
                });
            }
        }
        let mut column: Vec<Column> = Vec::new();
        for column_bytes in column_data {
            for identifier in &columns_identifiers {
                column.push(Column {
                    unique_identifier: *identifier,
                    data: column_bytes.to_string().into_bytes(),
                });
            }
        }
        let mut ident_name = name.clone();
        for char in ".near".chars() {
            ident_name.push(char);
        }
        Self {
            dao,
            name,
            owner,
            unique_identifier: AccountId::try_from(ident_name.to_lowercase()).unwrap(),
            row,
            column,
        }
    }

    pub fn default_registry() -> Self {
        Self {
            dao: "".to_string(),
            name: "".to_string(),
            owner: "some".to_string().parse().unwrap(),
            unique_identifier: "some1".to_string().parse().unwrap(),
            row: vec![],
            column: vec![],
        }
    }
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn init(owner_id: AccountId, dao: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner: owner_id.clone(),
            locked_amount: 0,
            staking_id: Some(owner_id),
            total_delegation_amount: 0,
            delegations: LookupMap::new(b"d".to_vec()),
            last_proposal_id: 0,
            proposals: TreeMap::new(b"p".to_vec()),
            registries: HashMap::new(),
            dao,
            voting_power: HashMap::new(),
        }
    }

    pub fn change_dao(&mut self, dao: AccountId) {
        self.dao = dao;
    }

    pub fn new_registry(
        &mut self,
        owner_id: AccountId,
        column_data: Vec<Value>,
        row_data: Vec<Value>,
        name: String,
    ) -> Value {
        let mut columns_identifiers = Vec::new();
        let mut rows_identifiers = Vec::new();
        if self.is_name_exist(name.clone()) {
            env::panic_str("That`s name already exist");
        }
        for column in &column_data {
            columns_identifiers
                .push(self.check_identifier(column.to_string().as_bytes().len() as u64));
        }
        for row in &row_data {
            rows_identifiers.push(self.check_identifier(row.to_string().as_bytes().len() as u64));
        }
        let mut vec = Vec::new();
        let registry_data = RegistryData::new(
            name,
            row_data,
            column_data,
            owner_id.clone(),
            self.dao.to_string(),
            columns_identifiers,
            rows_identifiers,
        );
        vec.push(registry_data.clone());
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.registries.entry(owner_id.clone())
        {
            e.insert(vec);
        } else {
            self.registries
                .entry(owner_id)
                .and_modify(|x| x.push(registry_data.clone()));
        }
        self.get_registry_by_name(registry_data.name)
    }

    pub fn change_registry(
        &mut self,
        unique_identifier: AccountId,
        new_row_data: Vec<Value>,
        new_column_data: Vec<Value>,
    ) -> Value {
        let mut return_data = json!({});
        let _ = self
            .registries
            .clone()
            .iter()
            .map(|(_, registry_data)| {
                for data in registry_data {
                    if data.unique_identifier == unique_identifier {
                        let mut columns_identifiers = Vec::new();
                        for identifier in &data.column {
                            columns_identifiers.push(identifier.unique_identifier)
                        }
                        let mut rows_identifiers = Vec::new();
                        for identifier in &data.row {
                            rows_identifiers.push(identifier.unique_identifier)
                        }
                        let new_data = RegistryData::new(
                            data.name.clone(),
                            new_row_data.clone(),
                            new_column_data.clone(),
                            data.owner.clone(),
                            data.dao.clone(),
                            columns_identifiers,
                            rows_identifiers,
                        );
                        let mut value = 0;
                        return_data = self.get_registry_by_name(data.name.clone());
                        self.registries.entry(data.owner.clone()).and_modify(|x| {
                            for i in x.clone() {
                                if i.unique_identifier == unique_identifier {
                                    x.remove(value);
                                    x.insert(0, new_data.clone());
                                    break;
                                } else {
                                    value += 1;
                                }
                            }
                        });
                    }
                }
            })
            .collect::<()>();
        return_data
    }

    pub fn delete_registry(&mut self, name: String) {
        for i in self.registries.clone().iter() {
            for data in i.1 {
                if data.name == name {
                    self.registries.remove(i.0);
                }
            }
        }
    }

    #[private]
    pub fn voting_change_registry(
        &mut self,
        unique_identifier: AccountId,
        new_row_data: Vec<Value>,
        new_column_data: Vec<Value>,
    ) {
        let _ = self
            .registries
            .clone()
            .iter()
            .map(|(_, registry_data)| {
                for data in registry_data {
                    if data.unique_identifier == unique_identifier {
                        let mut columns_identifiers = Vec::new();
                        for identifier in &data.column {
                            columns_identifiers.push(identifier.unique_identifier)
                        }
                        let mut rows_identifiers = Vec::new();
                        for identifier in &data.row {
                            rows_identifiers.push(identifier.unique_identifier)
                        }
                        let new_data = RegistryData::new(
                            data.name.clone(),
                            new_row_data.clone(),
                            new_column_data.clone(),
                            data.owner.clone(),
                            data.dao.clone(),
                            columns_identifiers,
                            rows_identifiers,
                        );
                        let mut value = 0;
                        self.registries.entry(data.owner.clone()).and_modify(|x| {
                            for i in x.clone() {
                                if i.unique_identifier == unique_identifier {
                                    x.remove(value);
                                    x.insert(0, new_data.clone());
                                    break;
                                } else {
                                    value += 1;
                                }
                            }
                        });
                    }
                }
            })
            .collect::<()>();
    }

    #[private]
    pub fn callback_promise_result(&mut self) -> i32 {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(result) = near_sdk::serde_json::from_slice::<i32>(&val) {
                    if result == 1 {
                        result
                    } else {
                        env::panic_str("BALANCE_DONT_CHANGE")
                    }
                } else {
                    env::panic_str("ERR_WRONG_VAL_RECEIVED")
                }
            }
            PromiseResult::Failed => env::panic_str("ERR_CALL_FAILED"),
        }
    }

    #[private]
    pub fn callback_add_proposal_result(&mut self) -> u64 {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(result) = near_sdk::serde_json::from_slice::<u64>(&val) {
                    if let Some(proposal) = self.proposals.get(&self.last_proposal_id) {
                        self.proposals.remove(&self.last_proposal_id);
                        self.proposals.insert(&result, &proposal);
                    };
                    self.last_proposal_id = result;
                    self.last_proposal_id
                } else {
                    env::panic_str("ERR_WRONG_VAL_RECEIVED")
                }
            }
            PromiseResult::Failed => env::panic_str("ERR_CALL_FAILED"),
        }
    }

    #[private]
    pub fn proposal_result_callback(&mut self) -> (ProposalOutput, ProposalStatus) {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(value) = near_sdk::serde_json::from_slice::<Value>(&val) {
                    let result: ProposalOutput = serde_json::from_value(value).unwrap();
                    match result.status {
                        ProposalStatus::InProgress => env::panic_str("PROPOSAL_IN_PROGRESS"),
                        ProposalStatus::Approved => {
                            let mut proposal: Proposal = self
                                .proposals
                                .get(&result.id.clone())
                                .expect("ERR_NO_PROPOSAL")
                                .into();
                            let values = self.get_registry(proposal.clone());
                            self.voting_change_registry(
                                self.get_identifier(proposal.new_registries.clone()),
                                vec![values.0],
                                vec![values.1],
                            );
                            self.internal_callback_proposal_success(&mut proposal);
                            (result.clone(), result.status)
                        }
                        ProposalStatus::Rejected => {
                            let mut proposal: Proposal = self
                                .proposals
                                .get(&result.id.clone())
                                .expect("ERR_NO_PROPOSAL")
                                .into();
                            self.internal_callback_proposal_fail(&mut proposal);
                            (result.clone(), result.status)
                        }
                        ProposalStatus::Removed => {
                            let mut proposal: Proposal = self
                                .proposals
                                .get(&result.id)
                                .expect("ERR_NO_PROPOSAL")
                                .into();
                            self.internal_callback_proposal_fail(&mut proposal);
                            (result.clone(), result.status)
                        }
                        ProposalStatus::Expired => {
                            let mut proposal: Proposal = self
                                .proposals
                                .get(&result.id)
                                .expect("ERR_NO_PROPOSAL")
                                .into();
                            self.internal_callback_proposal_fail(&mut proposal);
                            (result.clone(), result.status)
                        }
                        ProposalStatus::Moved => unreachable!(),
                        ProposalStatus::Failed => {
                            let mut proposal: Proposal = self
                                .proposals
                                .get(&result.id)
                                .expect("ERR_NO_PROPOSAL")
                                .into();
                            self.internal_callback_proposal_fail(&mut proposal);
                            (result.clone(), result.status)
                        }
                    }
                } else {
                    env::panic_str("ERR_WRONG_VAL_RECEIVED")
                }
            }
            PromiseResult::Failed => env::panic_str("ERR_CALL_FAILED"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::serde::export::TryFrom;
    use near_sdk::serde_json::json;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;
    use std::str::FromStr;

    fn alice() -> AccountId {
        AccountId::try_from("alice.near".to_string()).unwrap()
    }

    fn bob() -> AccountId {
        AccountId::try_from("bob.near".to_string()).unwrap()
    }

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    // mark individual unit tests with #[test] for them to be registered and fired
    // #[test]
    // fn get_registries() {
    //     // set up the mock context into the testing environment
    //     let context = get_context(alice());
    //     testing_env!(context.build());
    //     // instantiate a contract variable with the counter at zero
    //     let mut contract = Contract {
    //         owner: alice(),
    //         locked_amount: 0,
    //         staking_id: None,
    //         total_delegation_amount: 0,
    //         delegations: LookupMap::new(b"d"),
    //         last_proposal_id: 0,
    //         proposals: LookupMap::new(b"p"),
    //         registries: Default::default(),
    //         dao: bob(),
    //         voting_power: HashMap::new(),
    //     };
    //
    //     let value = json!({"id":"2489651045","type":"CreateEvent"});
    //     let value2 = json!({"dao": "", "name": "TestName", "owner": "alice.near", "unique_identifier": "testname.near"});
    //     contract.new_registry(alice(), value.clone(), value.clone(), "TestName".to_string());
    //     let result = contract.get_registries();
    //
    //     // println!("{:?}", value);
    //
    //     // println!("{:?}", result);
    //     // let mut vec = Vec::new();
    //     // vec.push(value);
    //
    //     assert_eq!(value, result[2]);
    //     assert_eq!(value2, result[0]);
    //     // confirm that we received 1 when calling get_num
    // }

    #[test]
    fn voting_change_registry() {
        // set up the mock context into the testing environment
        let context = get_context(alice());
        testing_env!(context.build());
        // instantiate a contract variable with the counter at zero
        let mut contract = Contract {
            owner: alice(),
            locked_amount: 0,
            staking_id: None,
            total_delegation_amount: 0,
            delegations: LookupMap::new(b"d"),
            last_proposal_id: 0,
            proposals: TreeMap::new(b"p"),
            registries: Default::default(),
            dao: bob(),
            voting_power: HashMap::new(),
        };

        let value = json!({"id":"2489651045","type":"CreateEvent"});
        let _value2 = json!({"dao": "", "name": "testname", "owner": "alice.near", "unique_identifier": "testname.near"});
        contract.new_registry(
            alice(),
            vec![value.clone(), value.clone()],
            vec![value.clone()],
            "testname".to_string(),
        );

        let new_value = json!({"id":"4647576214","type":"CreateEvent"});
        let _new_value2 = json!({"dao": "", "name": "testnamenew", "owner": "alice.near", "unique_identifier": "testnamenew.near"});
        contract.new_registry(
            alice(),
            vec![new_value.clone()],
            vec![new_value.clone()],
            "testnamenew".to_string(),
        );
        let result = contract.get_registries();
        println!("{:?}", result);
    }

    #[test]
    fn test_voting_change_registry() {
        // set up the mock context into the testing environment
        let context = get_context(alice());
        testing_env!(context.build());
        // instantiate a contract variable with the counter at zero
        let mut contract = Contract {
            owner: alice(),
            locked_amount: 0,
            staking_id: None,
            total_delegation_amount: 0,
            delegations: LookupMap::new(b"d"),
            last_proposal_id: 0,
            proposals: TreeMap::new(b"p"),
            registries: Default::default(),
            dao: bob(),
            voting_power: HashMap::new(),
        };

        let value = json!({"fruit": "Apple","size": "Large","color": "Red"});
        contract.new_registry(
            alice(),
            vec![value.clone()],
            vec![value.clone()],
            "testname".to_string(),
        );

        // let result = contract.get_registries();
        // println!("{:?}", result);

        // assert_eq!(value, result[1]);

        let value2 = json!({"fruit": "Banana","size": "Large","color": "Yellow"});
        contract.new_registry(
            alice(),
            vec![value2.clone()],
            vec![value2.clone()],
            "testnamenew".to_string(),
        );

        let value3 = json!({"fruit": "Cherry","size": "Small","color": "Red"});
        contract.new_registry(
            alice(),
            vec![value3.clone()],
            vec![value3.clone()],
            "testnamenew1".to_string(),
        );

        let _result = contract.get_registries();

        let new_value = json!({"fruit": "Pineapple","size": "Medium","color": "Yellow"});

        contract.voting_change_registry(
            AccountId::from_str("testname.near").unwrap(),
            vec![new_value.clone()],
            vec![new_value.clone()],
        );

        let new_result = contract.get_registries();

        println!("{:?}", new_result);
    }

    // #[test]
    // #[should_panic]
    // fn is_name_exist() {
    //     // set up the mock context into the testing environment
    //     let context = get_context(alice());
    //     testing_env!(context.build());
    //     // instantiate a contract variable with the counter at zero
    //     let mut contract = Contract {
    //         owner: alice(),
    //         locked_amount: 0,
    //         staking_id: None,
    //         total_delegation_amount: 0,
    //         delegations: LookupMap::new(b"d"),
    //         last_proposal_id: 0,
    //         proposals: LookupMap::new(b"p"),
    //         registries: Default::default(),
    //         dao: bob(),
    //         voting_power: HashMap::new(),
    //     };
    //
    //     let value = json!({"fruit": "Apple","size": "Large","color": "Red"});
    //     contract.new_registry(
    //         alice(),
    //         value.clone(),
    //         value.clone(),
    //         "TestName".to_string(),
    //     );
    //
    //     let result = contract.get_registries();
    //
    //     assert_eq!(value, result[1]);
    //
    //     let value2 = json!({"fruit": "Banana","size": "Large","color": "Yellow"});
    //     contract.new_registry(
    //         alice(),
    //         value2.clone(),
    //         value2.clone(),
    //         "TestName".to_string(),
    //     );
    // }

    // pub fn to_yocto(value: &str) -> u128 {
    //     let vals: Vec<_> = value.split('.').collect();
    //     let part1 = vals[0].parse::<u128>().unwrap() * 10u128.pow(24);
    //     if vals.len() > 1 {
    //         let power = vals[1].len() as u32;
    //         let part2 = vals[1].parse::<u128>().unwrap() * 10u128.pow(24 - power);
    //         part1 + part2
    //     } else {
    //         part1
    //     }
    // }
    //
    // fn create_proposal(
    //     context: &mut VMContextBuilder,
    //     contract: &mut Contract,
    //     data: Value,
    // ) -> u64 {
    //     testing_env!(context.attached_deposit(to_yocto("1")).build());
    //     contract.add_proposal(ProposalInput {
    //         owner: alice(),
    //         description: "test".to_string(),
    //         kind: ProposalKind::Vote,
    //         column: data.clone(),
    //         unique_identifier: AccountId::from_str("testname.near").unwrap(),
    //         row: data.clone(),
    //     })
    // }

    // #[test]
    // fn test_basics() {
    //     let mut context = VMContextBuilder::new();
    //     testing_env!(context.predecessor_account_id(accounts(0)).build());
    //     let mut contract = Contract {
    //         owner: alice(),
    //         locked_amount: 0,
    //         staking_id: None,
    //         total_delegation_amount: 0,
    //         delegations: LookupMap::new(b"d"),
    //         last_proposal_id: 0,
    //         proposals: LookupMap::new(b"p"),
    //         registries: Default::default(),
    //         ft_token: bob(),
    //         astro_dao: bob(),
    //         voting_power: HashMap::new(),
    //     };
    //
    //     let value = json!({"fruit": "Apple","size": "Large","color": "Red"});
    //     contract.new_registry(alice(), value.clone(), "testname".to_string());
    //     let value2 = json!({"fruit": "Banana","size": "Large","color": "Yellow"});
    //
    //     let data = contract.get_registries();
    //
    //     let _registry_data = RegistryData {
    //         name: "testname".to_string(),
    //         owner: alice(),
    //         unique_identifier: AccountId::from_str("testname.near").unwrap(),
    //         data: data[1].to_string().into_bytes(),
    //     };
    //
    //     let id = create_proposal(&mut context, &mut contract, value);
    //     assert_eq!(contract.get_proposal(id).proposal.description, "test");
    //     assert_eq!(contract.get_proposals(0, 10).len(), 1);
    //
    //     testing_env!(context.predecessor_account_id(accounts(1)).build());
    //     let id = create_proposal(&mut context, &mut contract, value2.clone());
    //     contract.act_proposal(id, Action::VoteApprove, U128::from(10000u128));
    //     assert_eq!(
    //         contract.get_proposal(id).proposal.status,
    //         ProposalStatus::InProgress
    //     );
    //
    //     testing_env!(context.predecessor_account_id(accounts(2)).build());
    //     contract.act_proposal(id, Action::VoteReject, U128::from(100u128));
    //     assert_eq!(
    //         contract.get_proposal(id).proposal.status,
    //         ProposalStatus::InProgress
    //     );
    //
    //     testing_env!(context.predecessor_account_id(accounts(3)).build());
    //     contract.act_proposal(id, Action::VoteApprove, U128::from(100000u128));
    //     assert_eq!(
    //         contract.get_proposal(id).proposal.status,
    //         ProposalStatus::InProgress
    //     );
    //
    //     testing_env!(context
    //         .block_timestamp(24 * 60 * 60 + 1)
    //         .predecessor_account_id(accounts(4))
    //         .build());
    //     contract.act_proposal(id, Action::VoteApprove, U128::from(1000u128));
    //     assert_eq!(contract.get_registries()[1], value2);
    //     assert_eq!(
    //         contract.get_proposal(id).proposal.status,
    //         ProposalStatus::Approved
    //     );
    // }

    // #[test]
    // pub fn check_voting() {
    //     let mut context = VMContextBuilder::new();
    //     testing_env!(context.predecessor_account_id(accounts(1)).build());
    //     let mut contract = Contract {
    //         owner: alice(),
    //         locked_amount: 0,
    //         staking_id: None,
    //         total_delegation_amount: 0,
    //         delegations: LookupMap::new(b"d"),
    //         last_proposal_id: 0,
    //         proposals: LookupMap::new(b"p"),
    //         registries: Default::default(),
    //         dao: bob(),
    //         voting_power: HashMap::new(),
    //     };
    //
    //     let value = json!({"fruit": "Apple","size": "Large","color": "Red"});
    //     contract.new_registry(
    //         alice(),
    //         value.clone(),
    //         value.clone(),
    //         "testname".to_string(),
    //     );
    //     let value2 = json!({"fruit": "Banana","size": "Large","color": "Yellow"});
    //
    //     let data = contract.get_registries();
    //
    //     let mut row = HashMap::new();
    //     let mut column = HashMap::new();
    //     row.insert(alice(), data[1].to_string().into_bytes());
    //     column.insert(alice(), data[1].to_string().into_bytes());
    //
    //     let _registry_data = RegistryData {
    //         dao: "".to_string(),
    //         name: "testname".to_string(),
    //         owner: alice(),
    //         unique_identifier: AccountId::from_str("testname.near").unwrap(),
    //         row: Row {
    //             unique_identifier: AccountId::from_str("testrow.near").unwrap(),
    //             data: data[1].to_string().into_bytes(),
    //         },
    //         column: Column {
    //             unique_identifier: AccountId::from_str("testcolumn.near").unwrap(),
    //             data: data[1].to_string().into_bytes(),
    //         },
    //     };
    //
    //     let id = create_proposal(&mut context, &mut contract, value);
    //     assert_eq!(contract.get_proposal(id).proposal.description, "test");
    //     assert_eq!(contract.get_proposals(0, 10).len(), 1);
    //
    //     let id = create_proposal(&mut context, &mut contract, value2.clone());
    //     contract.act_proposal(id, Action::VoteApprove, U128::from(1u128));
    //     let proposal = contract.get_proposal(id);
    //
    //     assert_eq!(proposal.proposal.status, ProposalStatus::InProgress);
    // }

    // #[test]
    // fn test_reject() {
    //     let mut context = VMContextBuilder::new();
    //     testing_env!(context.predecessor_account_id(accounts(0)).build());
    //     let mut contract = Contract {
    //         owner: alice(),
    //         locked_amount: 0,
    //         staking_id: None,
    //         total_delegation_amount: 0,
    //         delegations: LookupMap::new(b"d"),
    //         last_proposal_id: 0,
    //         proposals: LookupMap::new(b"p"),
    //         registries: Default::default(),
    //         ft_token: bob(),
    //         astro_dao: bob(),
    //         voting_power: HashMap::new(),
    //     };
    //
    //     let value = json!({"fruit": "Apple","size": "Large","color": "Red"});
    //     contract.new_registry(alice(), value.clone(), "testname".to_string());
    //     let value2 = json!({"fruit": "Banana","size": "Large","color": "Yellow"});
    //
    //     let data = contract.get_registries();
    //
    //     let _registry_data = RegistryData {
    //         name: "testname".to_string(),
    //         owner: alice(),
    //         unique_identifier: AccountId::from_str("testname.near").unwrap(),
    //         data: data[1].to_string().into_bytes(),
    //     };
    //
    //     let id = create_proposal(&mut context, &mut contract, value);
    //     assert_eq!(contract.get_proposal(id).proposal.description, "test");
    //     assert_eq!(contract.get_proposals(0, 10).len(), 1);
    //
    //     testing_env!(context.predecessor_account_id(accounts(1)).build());
    //     let id = create_proposal(&mut context, &mut contract, value2.clone());
    //     contract.act_proposal(id, Action::VoteApprove, U128::from(10000u128));
    //     assert_eq!(
    //         contract.get_proposal(id).proposal.status,
    //         ProposalStatus::InProgress
    //     );
    //
    //     testing_env!(context.predecessor_account_id(accounts(2)).build());
    //     contract.act_proposal(id, Action::VoteReject, U128::from(100000u128));
    //     assert_eq!(
    //         contract.get_proposal(id).proposal.status,
    //         ProposalStatus::InProgress
    //     );
    //
    //     testing_env!(context.predecessor_account_id(accounts(3)).build());
    //     contract.act_proposal(id, Action::VoteApprove, U128::from(100u128));
    //     assert_eq!(
    //         contract.get_proposal(id).proposal.status,
    //         ProposalStatus::InProgress
    //     );
    //
    //     testing_env!(context.predecessor_account_id(accounts(4)).build());
    //     testing_env!(context.block_timestamp(24 * 60 * 60 + 1).build());
    //     contract.act_proposal(id, Action::VoteApprove, U128::from(1000u128));
    //     assert_eq!(
    //         contract.get_proposal(id).proposal.status,
    //         ProposalStatus::Rejected
    //     );
    // }
}
