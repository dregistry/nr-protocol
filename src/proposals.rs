use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::{json_types::U128, AccountId, Balance, Promise, PromiseOrValue, PromiseResult};

use crate::{
    consts::{GAS_FOR_FT_TRANSFER, GAS_FOR_RESOLVE_TRANSFER, NO_DEPOSIT, ONE_NEAR, ONE_YOCTO_NEAR},
    types::{
        Action, Proposal, ProposalInput, ProposalInputAstroDao, ProposalStatus, VersionedProposal,
    },
    *,
};

impl Contract {
    /// Execute payout of given token to given user.
    #[allow(dead_code)]
    pub(crate) fn internal_payout(
        &mut self,
        token_id: &Option<AccountId>,
        receiver_id: &AccountId,
        amount: Balance,
        memo: String,
        msg: &Option<String>,
    ) -> PromiseOrValue<()> {
        if token_id.is_none() {
            Promise::new(receiver_id.clone()).transfer(amount).into()
        } else {
            if let Some(msg) = msg.clone() {
                ext_fungible_token::ft_transfer_call(
                    receiver_id.clone(),
                    U128(amount),
                    Some(memo),
                    msg,
                    token_id.as_ref().unwrap().clone(),
                    ONE_YOCTO_NEAR,
                    GAS_FOR_FT_TRANSFER,
                )
            } else {
                ext_fungible_token::ft_transfer(
                    receiver_id.clone(),
                    U128(amount),
                    Some(memo),
                    token_id.as_ref().unwrap().clone(),
                    ONE_YOCTO_NEAR,
                    GAS_FOR_FT_TRANSFER,
                )
            }
            .into()
        }
    }

    pub(crate) fn internal_callback_proposal_success(
        &mut self,
        proposal: &mut Proposal,
    ) -> PromiseOrValue<()> {
        proposal.status = ProposalStatus::Approved;
        PromiseOrValue::Value(())
    }

    pub(crate) fn internal_callback_proposal_fail(
        &mut self,
        proposal: &mut Proposal,
    ) -> PromiseOrValue<()> {
        proposal.status = ProposalStatus::Failed;
        PromiseOrValue::Value(())
    }
}

#[near_bindgen]
impl Contract {
    /// Add proposal to this DAO.
    #[payable]
    pub fn add_proposal(&mut self, proposal: ProposalInput) -> u64 {
        // 0. validate bond attached.
        let dao_proposal = ProposalInputAstroDao {
            description: proposal.description.clone(),
            kind: proposal.kind.clone(),
        };
        let _ = Promise::new(self.dao.clone())
            .function_call(
                "add_proposal".to_string(),
                json!({ "proposal": dao_proposal })
                    .to_string()
                    .as_bytes()
                    .to_vec(),
                ONE_NEAR,
                GAS_FOR_FT_TRANSFER,
            )
            .then(ext_self::callback_add_proposal_result(
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_FOR_RESOLVE_TRANSFER,
            ));

        // 3. Actually add proposal to the current list of proposals.

        self.proposals.insert(
            &self.last_proposal_id,
            &VersionedProposal::Default(proposal.into()),
        );
        self.locked_amount += env::attached_deposit();
        self.last_proposal_id
    }

    /// Act on given proposal by id, if permissions allow.
    /// Memo is logged but not stored in the state. Can be used to leave notes or explain the action.
    pub fn act_proposal(&mut self, id: u64, action: Action, amount: U128) {
        let _ = Promise::new(self.dao.clone()).function_call(
            "act_proposal".to_string(),
            json!({ "id": id, "action": action, "amount": amount })
                .to_string()
                .as_bytes()
                .to_vec(),
            NO_DEPOSIT,
            GAS_FOR_FT_TRANSFER,
        );
    }

    pub fn delete_proposal(&mut self, id: u64) {
        self.proposals.remove(&id);
    }

    /// Receiving callback after the proposal has been finalized.
    /// If successful, returns bond money to the proposal originator.
    /// If the proposal execution failed (funds didn't transfer or function call failure),
    /// move proposal to "Failed" state.
    #[private]
    pub fn on_proposal_callback(&mut self, proposal_id: u64) -> PromiseOrValue<()> {
        let mut proposal: Proposal = self
            .proposals
            .get(&proposal_id)
            .expect("ERR_NO_PROPOSAL")
            .into();
        assert_eq!(
            env::promise_results_count(),
            1,
            "ERR_UNEXPECTED_CALLBACK_PROMISES"
        );
        let result = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => self.internal_callback_proposal_success(&mut proposal),
            PromiseResult::Failed => self.internal_callback_proposal_fail(&mut proposal),
        };
        self.proposals
            .insert(&proposal_id, &VersionedProposal::Default(proposal));
        result
    }
}
