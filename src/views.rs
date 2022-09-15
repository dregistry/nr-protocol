use near_sdk::Promise;

use crate::consts::{GAS_FOR_RESOLVE_TRANSFER, NO_DEPOSIT};
use crate::types::{NewProposal, ProposalOutputRegistry};
use crate::*;

#[near_bindgen]
impl Contract {
    /// Last proposal's id.
    pub fn get_last_proposal_id(&self) -> u64 {
        self.last_proposal_id
    }

    // Get proposals in paginated view.
    // pub fn get_proposals(&self, from_index: u64, limit: u64) -> Vec<Value> {
    //     (from_index..min(self.last_proposal_id, from_index + limit))
    //         .filter_map(|id| {
    //             self.proposals
    //                 .get(&id)
    //                 .map(|proposal| {
    //                     let proposal = self.proposals.get( & id).expect("ERR_NO_PROPOSAL");
    //                     let mut vec = Vec::new();
    //                     let new_proposal: ProposalOutputRegistry = ProposalOutputRegistry {
    //                         id,
    //                         proposal: proposal.into(),
    //                     };
    //                     vec.push(serde_json::to_value(new_proposal.clone()).unwrap());
    //                     for row in & new_proposal.proposal.row {
    //                     vec.push(serde_json::to_value(row.unique_identifier).unwrap());
    //                     vec.push(serde_json::from_slice( & row.data).unwrap());
    //                 }
    //                      for column in &new_proposal.proposal.column {
    //                          vec.push(serde_json::to_value(column.unique_identifier).unwrap());
    //                          vec.push(serde_json::from_slice(&column.data).unwrap());
    //                      };
    //             vec
    //         })
    //         })
    //         .collect()
    // }

    pub fn get_all_proposals(&self) -> Vec<Value> {
        let mut vec = Vec::new();
        for proposal in self.proposals.iter() {
            let new_proposal: NewProposal = proposal.1.into();
            vec.push(json!({"proposal_id": proposal.0}));
            vec.push(serde_json::to_value(new_proposal.clone()).unwrap());
            for row in &new_proposal.row {
                vec.push(json!({"row_unique_identifier": row.unique_identifier}));
                let data: Value = serde_json::from_slice(&row.data).unwrap();
                vec.push(json!({ "row_data": data }));
            }
            for column in &new_proposal.column {
                vec.push(json!({"column_unique_identifier": column.unique_identifier}));
                let data: Value = serde_json::from_slice(&column.data).unwrap();
                vec.push(json!({ "column_data": data }));
            }
        }
        vec
    }

    /// Get specific proposal.
    pub fn get_proposal(&self, id: u64) -> Vec<Value> {
        let proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL");
        let mut vec = Vec::new();
        let new_proposal: ProposalOutputRegistry = ProposalOutputRegistry {
            id,
            proposal: proposal.into(),
        };
        vec.push(serde_json::to_value(new_proposal.clone()).unwrap());
        for row in &new_proposal.proposal.row {
            vec.push(serde_json::to_value(row.unique_identifier).unwrap());
            vec.push(serde_json::from_slice(&row.data).unwrap());
        }
        for column in &new_proposal.proposal.column {
            vec.push(serde_json::to_value(column.unique_identifier).unwrap());
            vec.push(serde_json::from_slice(&column.data).unwrap());
        }
        vec
    }

    pub fn get_voting_result(&self, proposal_id: u64) -> Vec<Value> {
        let _ = Promise::new(self.dao.clone())
            .function_call(
                "get_proposal".to_string(),
                json!({ "id": proposal_id }).to_string().as_bytes().to_vec(),
                NO_DEPOSIT,
                GAS_FOR_RESOLVE_TRANSFER,
            )
            .then(ext_self::proposal_result_callback(
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_FOR_RESOLVE_TRANSFER,
            ));
        self.get_proposal(proposal_id)
    }

    pub fn get_registries(&self) -> Vec<Value> {
        let mut vec = Vec::new();
        for (_account, registry) in self.registries.iter() {
            for data in registry {
                let registry_data = serde_json::to_value(&data).unwrap();
                let mut column_data = Vec::new();
                for column in &data.column {
                    let new_data: Value = serde_json::from_slice(&column.data).unwrap();
                    let column_identifier: Value =
                        serde_json::to_value(column.unique_identifier).unwrap();
                    column_data.push(column_identifier);
                    column_data.push(new_data);
                }
                let mut row_data = Vec::new();
                for row in &data.row {
                    let new_data: Value = serde_json::from_slice(&row.data).unwrap();
                    let row_identifier: Value =
                        serde_json::to_value(row.unique_identifier).unwrap();
                    row_data.push(row_identifier);
                    row_data.push(new_data);
                }
                vec.push(json!({"Registry": registry_data, "column_data": column_data, "row_data": row_data}));
            }
        }
        vec
    }

    pub fn get_registry_by_owner(&self, owner: AccountId) -> Option<&Vec<RegistryData>> {
        self.registries.get(&owner)
    }

    #[private]
    pub fn is_name_exist(&mut self, name: String) -> bool {
        let mut value = false;
        for (_, registry) in self.registries.iter() {
            for data in registry {
                if data.name == *name {
                    value = true;
                }
            }
        }
        value
    }

    #[private]
    pub fn get_registry_by_name(&mut self, name: String) -> Value {
        let mut value: Value = json!({});
        for (_account, registry) in self.registries.iter() {
            for data in registry {
                if data.name == name {
                    let registry_data = serde_json::to_value(&data).unwrap();
                    let mut column_data = Vec::new();
                    for column in &data.column {
                        let new_data: Value = serde_json::from_slice(&column.data).unwrap();
                        let column_identifier: Value =
                            serde_json::to_value(column.unique_identifier).unwrap();
                        column_data.push(column_identifier);
                        column_data.push(new_data);
                    }
                    let mut row_data = Vec::new();
                    for row in &data.row {
                        let new_data: Value = serde_json::from_slice(&row.data).unwrap();
                        let row_identifier: Value =
                            serde_json::to_value(row.unique_identifier).unwrap();
                        row_data.push(row_identifier);
                        row_data.push(new_data);
                    }
                    value = json!({"Registry": registry_data, "column_data": column_data, "row_data": row_data});
                }
            }
        }
        value
    }

    #[private]
    pub fn get_identifier(
        &self,
        hash_map: HashMap<AccountId, Vec<(AccountId, Vec<u8>)>>,
    ) -> AccountId {
        let mut uuid = AccountId::new_unchecked("newvalue".to_string());
        for (_, registries) in self.registries.iter() {
            for data in registries {
                for (_, registry) in hash_map.iter() {
                    for id in registry {
                        if id.0 == data.unique_identifier {
                            uuid = data.unique_identifier.clone();
                        }
                    }
                }
            }
        }
        uuid
    }

    #[private]
    pub fn get_registry(&self, proposal: Proposal) -> (Value, Value) {
        let mut value = json!("");
        let mut value_1 = json!("");
        for (_, registries) in self.registries.iter() {
            for data in registries {
                for row in &proposal.new_rows {
                    let _ = data.row.iter().map(|x| {
                        if row.unique_identifier == x.unique_identifier {
                            let new_value: Value = serde_json::to_value(&x.clone()).unwrap();
                            value = new_value;
                        }
                    });
                }
                for column in &proposal.new_columns {
                    let _ = data.column.iter().map(|x| {
                        if column.unique_identifier == x.unique_identifier {
                            let new_value: Value = serde_json::to_value(&x.clone()).unwrap();
                            value_1 = new_value;
                        }
                    });
                }
            }
        }
        (value, value_1)
    }

    #[private]
    pub fn get_all_registries(&self) -> Vec<RegistryData> {
        let mut vec = Vec::new();
        for (_account, registry) in self.registries.iter() {
            for data in registry {
                vec.push(data.clone());
            }
        }
        vec
    }

    #[private]
    #[allow(unused_mut)]
    pub fn check_identifier(&self, mut id: u64) -> u64 {
        let registries = self.get_all_registries();
        for registry in registries {
            for row in registry.row {
                if row.unique_identifier == id {
                    id += 1;
                }
            }
            for column in registry.column {
                if column.unique_identifier == id {
                    id += 1;
                }
            }
        }
        id
    }
}
