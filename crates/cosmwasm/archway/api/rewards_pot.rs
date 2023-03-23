use std::num::NonZeroU128;

use archway_bindings::types::rewards::{RewardsRecordsResponse, WithdrawRewardsResponse};
use archway_bindings::{ArchwayMsg, ArchwayQuery, PageRequest};
use cosmwasm_std::{coins, Addr, BankMsg, Deps, DepsMut, Env, Reply as CwReply, SubMsg, Uint128};
use kv_storage::{MutStorage, Storage};

use referrals_core::rewards_pot::{HandleReply, Query};
use referrals_core::{FallibleApi, Id};
use referrals_cw::rewards_pot::{AdminResponse, DappResponse, TotalRewardsResponse};

pub use crate::{cache, Api, CwMutStore, CwStore, CwStoreError, Error as BaseApiError, Response};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API not initialized")]
    NotInitialized,
    #[error("dApp fee has not been set")]
    DappFeeNotSet,
    #[error("expected data in cosmwasm reply")]
    ExptectedReplyData,
    #[error("overflow adding collected rewards")]
    TotalCollectedOverflow,
    #[error("overflow adding total rewards")]
    TotalRewardsOverflow,
}

pub type ApiError<StoreError> = BaseApiError<StoreError, Error>;
pub type ApiResult<T, StoreError> = Result<T, ApiError<StoreError>>;
pub type CwApiError = ApiError<CwStoreError>;

#[derive(Default)]
pub struct RewardsPot;

#[must_use]
pub fn from_deps<'a>(
    deps: Deps<'a, ArchwayQuery>,
    env: &'a Env,
) -> Api<'a, RewardsPot, CwStore<'a>> {
    Api::new(CwStore::from_repo(deps.storage), env, deps.querier)
}

#[must_use]
pub fn from_deps_mut<'a>(
    deps: &'a mut DepsMut<ArchwayQuery>,
    env: &'a Env,
) -> Api<'a, RewardsPot, CwMutStore<'a>> {
    let deps = deps.branch();
    Api::new(CwMutStore::from_repo(deps.storage), env, deps.querier)
}

impl FallibleApi for RewardsPot {
    type Error = Error;
}

impl<'a, Store> Api<'a, RewardsPot, Store>
where
    Store: Storage,
{
    /// Query the rewards pots total rewards records count
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with the underlying querier.
    pub fn total_rewards_records(&self) -> ApiResult<u64, Store::Error> {
        let rewards_records_response: RewardsRecordsResponse = self.querier.query(
            &ArchwayQuery::rewards_records_with_pagination(
                &self.env.contract.address,
                PageRequest::new().limit(0),
            )
            .into(),
        )?;

        let total_records = rewards_records_response
            .pagination
            .and_then(|page| page.total)
            .unwrap_or_default();

        Ok(total_records)
    }

    /// Calculate how many outstanding rewards records the rewards pot has.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with the underlying querier or storage.
    pub fn outstanding_records(&self) -> ApiResult<u64, Store::Error> {
        let records_collected = cache::rewards_pot::reward_records_collected(&self.store)?;
        let total_records = self.total_rewards_records()?;
        let outstanding_records = total_records.saturating_sub(records_collected);

        Ok(outstanding_records)
    }

    /// The total amount of the rewards received and receivable by the rewards pot.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Calculating the total rewards overflows.
    /// - There is an issue with the underlying querier or storage.
    pub fn total_rewards(&self) -> ApiResult<TotalRewardsResponse, Store::Error> {
        let rewards_collected = cache::rewards_pot::total_rewards_collected(&self.store)?;

        let outstanding_records = self.outstanding_records()?;

        if outstanding_records == 0 {
            return Ok(TotalRewardsResponse {
                total: rewards_collected.into(),
            });
        }

        let rewards_records_response: RewardsRecordsResponse = self.querier.query(
            &ArchwayQuery::rewards_records_with_pagination(
                &self.env.contract.address,
                PageRequest::new().reverse().limit(outstanding_records),
            )
            .into(),
        )?;

        let total = rewards_records_response
            .records
            .into_iter()
            .flat_map(|record| record.rewards)
            .try_fold(rewards_collected, |total, reward| {
                total
                    .checked_add(reward.amount.u128())
                    .ok_or(Error::TotalRewardsOverflow)
            })
            .map(Uint128::from)
            .map_err(ApiError::Mode)?;

        Ok(TotalRewardsResponse { total })
    }

    /// The dApp associated with the pot
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - There is an issue with the underlying querier or storage.
    pub fn dapp(&self) -> ApiResult<DappResponse, Store::Error> {
        let dapp = cache::rewards_pot::dapp(&self.store)?
            .ok_or(Error::NotInitialized)
            .map_err(ApiError::Mode)?;

        Ok(DappResponse { dapp })
    }

    /// The admin of the pot
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - There is an issue with the underlying querier or storage.
    pub fn admin(&self) -> ApiResult<AdminResponse, Store::Error> {
        let admin = cache::rewards_pot::creator(&self.store)?
            .ok_or(Error::NotInitialized)
            .map_err(ApiError::Mode)?;

        Ok(AdminResponse { admin })
    }
}

impl<'a, Store> Api<'a, RewardsPot, Store>
where
    Store: MutStorage,
{
    /// Initialize the API so it can process `reward_pot::Reply`'s.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with underlying storage.
    pub fn initialize(&mut self, creator: Addr, dapp: &String) -> ApiResult<(), Store::Error> {
        cache::rewards_pot::set_creator(&mut self.store, &creator.into_string())?;
        cache::rewards_pot::set_dapp(&mut self.store, dapp)?;

        Ok(())
    }

    /// Handle a `WithdrawRewardsResponse` from issueing a `ArchwayMsg::WithdrawRewards` submessage
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Calculating the new total rewards collected overflows.
    pub fn handle_withdraw_rewards_response(
        &mut self,
        response: &WithdrawRewardsResponse,
    ) -> ApiResult<(), Store::Error> {
        let Some(rewards) = response.total_rewards.first() else {
            return Ok(());
        };

        let current_total_collected = cache::rewards_pot::total_rewards_collected(&self.store)?;

        let new_total_collected = current_total_collected
            .checked_add(rewards.amount.u128())
            .ok_or(Error::TotalCollectedOverflow)
            .map_err(ApiError::Mode)?;

        cache::rewards_pot::set_total_rewards_collected(&mut self.store, new_total_collected)?;

        let total_records = self.total_rewards_records()?;

        cache::rewards_pot::set_rewards_records_collected(&mut self.store, total_records)?;

        Ok(())
    }

    /// Handle a cosmwasm reply to a submessage
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - There is no data attached to the reply
    pub fn handle_cw_reply(&mut self, reply: CwReply) -> ApiResult<(), Store::Error> {
        let response = reply
            .result
            .into_result()
            .expect("submessages issued as reply_on_success");

        let data = response
            .data
            .ok_or(Error::ExptectedReplyData)
            .map_err(ApiError::Mode)?;

        let withdraw_response = cosmwasm_std::from_binary(&data)?;

        self.handle_withdraw_rewards_response(&withdraw_response)
    }
}

impl<'a, Store> FallibleApi for Api<'a, RewardsPot, Store>
where
    Store: Storage,
{
    type Error = ApiError<Store::Error>;
}

impl<'a, Store> HandleReply for Api<'a, RewardsPot, Store>
where
    Store: MutStorage,
{
    type Response = Response;

    fn into_response(self) -> Self::Response {
        self.response
    }

    fn withdraw_pending(&mut self) -> Result<(), Self::Error> {
        let outstanding_records = self.outstanding_records()?;

        if outstanding_records == 0 {
            return Ok(());
        }

        self.response.messages.push(SubMsg::reply_on_success(
            ArchwayMsg::withdraw_rewards_by_limit(outstanding_records),
            0, // the only reply_on submessage we send
        ));

        Ok(())
    }

    fn send_rewards(&mut self, receiver: Id, amount: NonZeroU128) -> Result<(), Self::Error> {
        let rewards_denom = self.rewards_denom()?;

        self.response.messages.push(SubMsg::new(BankMsg::Send {
            to_address: receiver.into_string(),
            amount: coins(amount.get(), rewards_denom),
        }));

        Ok(())
    }
}

impl<'a, Store> Query for Api<'a, RewardsPot, Store>
where
    Store: Storage,
{
    fn owner_id(&self) -> Result<Id, Self::Error> {
        cache::rewards_pot::creator(&self.store)?
            .ok_or(Error::NotInitialized)
            .map(Id::from)
            .map_err(ApiError::Mode)
    }

    fn has_uncollected_rewards(&self) -> Result<bool, Self::Error> {
        let outstanding_records = self.outstanding_records()?;
        Ok(outstanding_records > 0)
    }
}
