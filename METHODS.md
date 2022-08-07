## Methods, what user can call

### Initialization contract
```fn init(owner_id: AccountId, dao: AccountId)```
### in JSON like:

`````'{"owner_id": "some_account.testnet", "dao":"sputnikdao2.testnet"}'`````  
#
### Change dao contract for voting
```fn change_dao(dao: AccountId)```
### in JSON like
`````'{"dao":"sputnikdao2.testnet"}'`````

#
### Creating registry
```fn new_registry(dao: AccountId)```
### in JSON like:

`````'{'{"owner_id": "SOME_OWNER_FOR_THIS_REGISTRY", "column_data": '['{"some_value": "value"}']', "row_data": '['{"some_value": "value"}']', "name": "SOME_NAME"}'`````

#
### Change dao contract for voting
```fn change_dao(dao: AccountId)```
### in JSON like:

`````'{"dao":"sputnikdao2.testnet"}'`````

#
### Add proposal into dao contract
```
fn add_proposal(proposal: ProposalInput)

pub struct ProposalInput {
    pub owner: AccountId,
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// New registry
    pub column: Vec<Value>,
    pub row: Vec<Value>,
    /// UUID
    pub unique_identifier: AccountId,
}

```

### in JSON like:

`````'{"proposal": {"owner": "near_registry.testnet", "description": "Some description", "kind": "Vote", "column": '['{"unique_identifier": 29,"data": "value"}']', "row": '['{"unique_identifier": 29,"data": "value"}']', "unique_identifier": "test1.testnet"}}'`````

#
### Act for proposal (VoteApprove or VoteReject)
```act_proposal(id: u64, action: Action, amount: U128)```
### in JSON like:

`````'{"id": 0, "action": "VoteApprove", "amount": "10000000"}'`````

## View methods

### Get last proposal_id
```fn get_last_proposal_id()```

#
### Get proposal

```fn get_proposal(id: u64)```

### in JSON like:

```'{"id": 0}'```

#
### Get all registries

```fn get_registries()```

#
### Get registry by owner

```fn get_registry_by_owner(owner: AccountId)```

## in JSON like: 

```'{"owner": "OWNER_ACCOUNT"}'```