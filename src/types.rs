use crate::{consts::OLD_BASE_TOKEN, Column, Row};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::{Base64VecU8, U128, U64},
    serde::{Deserialize, Serialize},
    serde_json::Value,
    AccountId, Balance,
};
use std::{collections::HashMap, str::FromStr};

/// Set of possible action to take.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum Action {
    /// Action to add proposal. Used internally.
    AddProposal,
    /// Vote to approve given proposal or bounty.
    VoteApprove,
    /// Vote to reject given proposal or bounty.
    VoteReject,
}

impl Action {
    pub fn to_policy_label(&self) -> String {
        format!("{:?}", self)
    }
}

/// Status of a proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalStatus {
    InProgress,
    /// If quorum voted yes, this proposal is successfully approved.
    Approved,
    /// If quorum voted no, this proposal is rejected. Bond is returned.
    Rejected,
    /// If quorum voted to remove (e.g. spam), this proposal is rejected and bond is not returned.
    /// Interfaces shouldn't show removed proposals.
    Removed,
    /// Expired after period of time.
    Expired,
    /// If proposal was moved to Hub or somewhere else.
    Moved,
    /// If proposal has failed when finalizing. Allowed to re-finalize again to either expire or approved.
    Failed,
}

/// Function call arguments.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct ActionCall {
    method_name: String,
    args: Base64VecU8,
    deposit: U128,
    gas: U64,
}

/// Function call arguments.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct PolicyParameters {
    pub proposal_bond: Option<U128>,
    pub proposal_period: Option<U64>,
    pub bounty_bond: Option<U128>,
    pub bounty_forgiveness_period: Option<U64>,
}

/// Kinds of proposals, doing different action.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKind {
    /// Transfers given amount of `token_id` from this DAO to `receiver_id`.
    /// If `msg` is not None, calls `ft_transfer_call` with given `msg`. Fails if this base token.
    /// For `ft_transfer` and `ft_transfer_call` `memo` is the `description` of the proposal.
    Transfer {
        /// Can be "" for $NEAR or a valid account id.
        token_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
        msg: Option<String>,
    },
    /// Sets staking contract. Can only be proposed if staking contract is not set yet.
    SetStakingContract { staking_id: AccountId },
    /// Just a signaling vote, with no execution.
    Vote,
}

impl ProposalKind {
    /// Returns label of policy for given type of proposal.
    pub fn to_policy_label(&self) -> &str {
        match self {
            ProposalKind::Transfer { .. } => "transfer",
            ProposalKind::SetStakingContract { .. } => "set_vote_token",
            ProposalKind::Vote => "vote",
        }
    }
}

/// Votes recorded in the proposal.
#[derive(
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
)]
#[serde(crate = "near_sdk::serde")]
pub enum Vote {
    Approve = 0x0,
    Reject = 0x1,
    InProgress = 0x2,
}

impl From<Action> for Vote {
    fn from(action: Action) -> Self {
        match action {
            Action::VoteApprove => Vote::Approve,
            Action::VoteReject => Vote::Reject,
            _ => unreachable!(),
        }
    }
}

/// Proposal that are sent to this DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    /// Original proposer.
    pub proposer: AccountId,
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// Count of votes per role per decision: yes / no / spam.
    pub vote_counts: HashMap<String, [Balance; 3]>,
    /// Map of who voted and how.
    pub votes: HashMap<AccountId, Vote>,
    /// Submission time (for voting period).
    pub submission_time: U64,

    /// Proposals registries
    pub new_registries: HashMap<AccountId, Vec<(AccountId, Vec<u8>)>>,
    /// New column
    pub(crate) new_columns: Vec<Column>,
    /// New row
    pub(crate) new_rows: Vec<Row>,
    pub unique_identifier: AccountId,
}

/// Proposal that are sent to this DAO.
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct NewProposal {
    /// Original proposer.
    pub proposer: AccountId,
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// Count of votes per role per decision: yes / no / spam.
    pub vote_counts: HashMap<String, [Balance; 3]>,
    /// Map of who voted and how.
    pub votes: HashMap<AccountId, Vote>,
    /// Submission time (for voting period).
    pub submission_time: U64,
    /// Proposals registries
    pub row: Vec<Row>,
    pub column: Vec<Column>,
    pub unique_identifier: AccountId,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedProposal {
    Default(Proposal),
}

impl From<VersionedProposal> for Proposal {
    fn from(v: VersionedProposal) -> Self {
        match v {
            VersionedProposal::Default(p) => p,
        }
    }
}

impl From<VersionedProposal> for NewProposal {
    fn from(v: VersionedProposal) -> Self {
        match v {
            VersionedProposal::Default(p) => {
                let mut new_column = Vec::new();
                let mut new_row = Vec::new();
                for column in p.new_columns {
                    new_column.push(column);
                }
                for row in p.new_rows {
                    new_row.push(row);
                }
                Self {
                    proposer: p.proposer,
                    description: p.description,
                    kind: p.kind,
                    status: p.status,
                    vote_counts: p.vote_counts,
                    votes: p.votes,
                    submission_time: p.submission_time,
                    row: new_row,
                    column: new_column,
                    unique_identifier: p.unique_identifier,
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
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

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalInputAstroDao {
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
}

impl From<ProposalInput> for Proposal {
    fn from(input: ProposalInput) -> Self {
        let mut rows = Vec::new();
        for row in input.row {
            let row_data: Vec<u8> = row["data"].clone().to_string().as_bytes().to_vec();
            let row_identifier: u64 =
                serde_json::from_value(row["unique_identifier"].clone()).unwrap();
            rows.push(Row {
                unique_identifier: row_identifier,
                data: row_data,
            });
        }
        let mut columns = Vec::new();
        for column in input.column {
            let column_data: Vec<u8> = column["data"].clone().to_string().as_bytes().to_vec();
            let column_identifier: u64 =
                serde_json::from_value(column["unique_identifier"].clone()).unwrap();
            columns.push(Column {
                unique_identifier: column_identifier,
                data: column_data,
            });
        }
        Self {
            proposer: env::predecessor_account_id(),
            description: input.description,
            kind: input.kind,
            status: ProposalStatus::InProgress,
            vote_counts: HashMap::default(),
            votes: HashMap::default(),
            submission_time: U64::from(env::block_timestamp()),
            new_rows: rows,
            new_columns: columns,
            new_registries: Default::default(),
            unique_identifier: input.unique_identifier,
        }
    }
}

/// This is format of output via JSON for the proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalOutputRegistry {
    /// Id of the proposal.
    pub id: u64,
    #[serde(flatten)]
    pub proposal: NewProposal,
}

/// This is format of output via JSON for the proposal.
#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalOutput {
    /// Id of the proposal.
    pub id: u64,
    /// Original proposer.
    pub proposer: AccountId,
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// Count of votes per role per decision: yes / no / spam.
    pub vote_counts: HashMap<String, [Balance; 3]>,
    /// Map of who voted and how.
    pub votes: HashMap<AccountId, Vote>,
    /// Submission time (for voting period).
    pub submission_time: U64,
}
/// In near-sdk v3, the token was represented by a String, with no other restrictions.
/// That being said, Sputnik used "" (empty String) as a convention to represent the $NEAR token.
/// In near-sdk v4, the token representation was replaced by AccountId (which is in fact a wrapper
/// over a String), with the restriction that the token must be between 2 and 64 chars.
/// Sputnik had to adapt since "" was not allowed anymore and we chose to represent the token as a
/// Option<AccountId> with the convention that None represents the $NEAR token.
/// This function is required to help with the transition and keep the backward compatibility.
#[allow(dead_code)]
pub fn convert_old_to_new_token(old_account_id: &AccountId) -> Option<AccountId> {
    if *old_account_id == AccountId::from_str(OLD_BASE_TOKEN).unwrap() {
        return None;
    }
    Some(AccountId::new_unchecked(old_account_id.to_string()))
}
