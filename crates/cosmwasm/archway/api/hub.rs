use std::num::NonZeroU128;

use archway_bindings::types::rewards::{ContractMetadataResponse, FlatFeeResponse};
use archway_bindings::{ArchwayMsg, ArchwayQuery};
use cosmwasm_std::{Coin, Deps, DepsMut, Env, SubMsg, WasmMsg};

use kv_storage::{MutStorage, Storage};

use referrals_core::hub::{
    CollectQuery, DappQuery, HandleReply, MutableCollectStore, MutableDappStore,
    MutableReferralStore, NonZeroPercent, ReadonlyCollectStore, ReadonlyDappStore,
    ReadonlyReferralStore, ReferralCode,
};
use referrals_core::{FallibleApi, Id};
use referrals_cw::rewards_pot::{
    AdminResponse, ExecuteMsg as PotExecMsg, InstantiateMsg as PotInitMsg,
    QueryMsg as RewardsPotQuery, TotalRewardsResponse,
};
use referrals_cw::ReferralCodeResponse;

use crate::{cache, Api, CwMutStore, CwStore, CwStoreError, Error as BaseApiError, Response};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API not initialized")]
    NotInitialized,
    #[error("dApp fee has not been set")]
    DappFeeNotSet,
}

pub type ApiError<StoreError> = BaseApiError<StoreError, Error>;
pub type ApiResult<T, StoreError> = Result<T, ApiError<StoreError>>;
pub type CwApiError = ApiError<CwStoreError>;

#[derive(Default)]
pub struct Hub;

#[must_use]
pub fn from_deps<'a>(deps: Deps<'a, ArchwayQuery>, env: &'a Env) -> Api<'a, Hub, CwStore<'a>> {
    Api::new(CwStore::from_repo(deps.storage), env, deps.querier)
}

#[must_use]
pub fn from_deps_mut<'a>(
    deps: &'a mut DepsMut<ArchwayQuery>,
    env: &'a Env,
) -> Api<'a, Hub, CwMutStore<'a>> {
    let deps = deps.branch();
    Api::new(CwMutStore::from_repo(deps.storage), env, deps.querier)
}

impl FallibleApi for Hub {
    type Error = Error;
}

impl<'a, Store> Api<'a, Hub, Store>
where
    Store: MutStorage,
{
    /// Initialize the API so it can process `hub::Reply`'s.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with underlying storage.
    pub fn initialize(&mut self, rewards_pot_code_id: u64) -> ApiResult<(), Store::Error> {
        cache::hub::set_reward_pot_code_id(&mut self.store, rewards_pot_code_id)?;
        Ok(())
    }
}

impl<'a, Store> FallibleApi for Api<'a, Hub, Store>
where
    Store: Storage,
{
    type Error = ApiError<Store::Error>;
}

impl<'a, Store> HandleReply for Api<'a, Hub, Store>
where
    Store: MutStorage,
{
    type Response = Response;

    fn into_response(self) -> Self::Response {
        self.response
    }

    fn add_referral_code(&mut self, code: ReferralCode) -> Result<(), Self::Error> {
        let data = cosmwasm_std::to_binary(&ReferralCodeResponse {
            code: code.to_u64(),
        })?;

        self.response.data = Some(data);

        Ok(())
    }

    fn create_rewards_pot(&mut self, dapp: Id) -> Result<(), Self::Error> {
        let code_id = cache::hub::reward_pot_code_id(&self.store)?
            .ok_or(Error::NotInitialized)
            .map_err(ApiError::Mode)?;

        let count = cache::hub::increment_reward_pot_count(&mut self.store)?;

        let msg = cosmwasm_std::to_binary(&PotInitMsg {
            dapp: dapp.into_string(),
        })?;

        self.response.messages.push(SubMsg::reply_on_success(
            WasmMsg::Instantiate {
                admin: None,
                code_id,
                msg,
                funds: vec![],
                label: format!("referrals-reward-pot-{count}"),
            },
            0, // only reply_on message we send
        ));

        Ok(())
    }

    fn set_rewards_recipient(&mut self, dapp: Id, recipient: Id) -> Result<(), Self::Error> {
        self.response
            .messages
            .push(SubMsg::new(ArchwayMsg::update_rewards_address(
                dapp.into_string(),
                recipient.into_string(),
            )));

        Ok(())
    }

    fn set_rewards_admin(&mut self, dapp: Id, admin: Id) -> Result<(), Self::Error> {
        self.response
            .messages
            .push(SubMsg::new(ArchwayMsg::update_rewards_ownership(
                dapp.into_string(),
                admin.into_string(),
            )));

        Ok(())
    }

    fn set_dapp_fee(&mut self, dapp: Id, amount: NonZeroU128) -> Result<(), Self::Error> {
        let denom = self.rewards_denom()?;
        self.response
            .messages
            .push(SubMsg::new(ArchwayMsg::set_flat_fee(
                dapp.into_string(),
                Coin::new(amount.get(), denom),
            )));

        Ok(())
    }

    fn withdraw_rewards(&mut self, pot: Id) -> Result<(), Self::Error> {
        let msg = cosmwasm_std::to_binary(&PotExecMsg::WithdrawRewards {})?;

        self.response.messages.push(SubMsg::new(WasmMsg::Execute {
            contract_addr: pot.into_string(),
            msg,
            funds: vec![],
        }));

        Ok(())
    }

    fn distribute_rewards(
        &mut self,
        pot: Id,
        amount: NonZeroU128,
        receiver: Id,
    ) -> Result<(), Self::Error> {
        let msg = cosmwasm_std::to_binary(&PotExecMsg::DistributeRewards {
            recipient: receiver.into_string(),
            amount: amount.get().into(),
        })?;

        self.response.messages.push(SubMsg::new(WasmMsg::Execute {
            contract_addr: pot.into_string(),
            msg,
            funds: vec![],
        }));

        Ok(())
    }
}

impl<'a, Store> DappQuery for Api<'a, Hub, Store>
where
    Store: Storage,
{
    fn self_id(&self) -> Result<Id, Self::Error> {
        Ok(Id::from(self.env.contract.address.clone()))
    }

    fn rewards_admin(&self, id: &Id) -> Result<Id, Self::Error> {
        let contract_metadata: ContractMetadataResponse = self
            .querier
            .query(&ArchwayQuery::contract_metadata(id.clone().into_string()).into())
            .map_err(ApiError::CosmWasmStd)?;

        Ok(Id::from(contract_metadata.owner_address))
    }

    fn rewards_pot_admin(&self, id: &Id) -> Result<Id, Self::Error> {
        let response: AdminResponse = self
            .querier
            .query_wasm_smart(id.clone().into_string(), &RewardsPotQuery::Admin {})
            .map_err(ApiError::CosmWasmStd)?;

        Ok(Id::from(response.admin))
    }

    fn current_fee(&self, id: &Id) -> Result<NonZeroU128, Self::Error> {
        let response: FlatFeeResponse = self
            .querier
            .query(&ArchwayQuery::flat_fee(id.as_str()).into())
            .map_err(ApiError::CosmWasmStd)?;

        NonZeroU128::new(response.flat_fee_amount.amount.u128())
            .ok_or(Error::DappFeeNotSet)
            .map_err(ApiError::Mode)
    }
}

impl<'a, Store> CollectQuery for Api<'a, Hub, Store>
where
    Store: Storage,
{
    fn dapp_total_rewards(&self, pot: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        let response: TotalRewardsResponse = self
            .querier
            .query_wasm_smart(pot.clone().into_string(), &RewardsPotQuery::TotalRewards {})
            .map_err(ApiError::CosmWasmStd)?;

        Ok(NonZeroU128::new(response.total.u128()))
    }
}

// Delegation to CoreStorage boilerplate

impl<'a, Store> ReadonlyDappStore for Api<'a, Hub, Store>
where
    Store: Storage,
{
    fn dapp_exists(&self, id: &Id) -> Result<bool, Self::Error> {
        self.core_storage().dapp_exists(id).map_err(ApiError::from)
    }

    fn percent(&self, id: &Id) -> Result<NonZeroPercent, Self::Error> {
        self.core_storage().percent(id).map_err(ApiError::from)
    }

    fn collector(&self, id: &Id) -> Result<Id, Self::Error> {
        self.core_storage().collector(id).map_err(ApiError::from)
    }

    fn has_rewards_pot(&self, id: &Id) -> Result<bool, Self::Error> {
        self.core_storage()
            .has_rewards_pot(id)
            .map_err(ApiError::from)
    }

    fn rewards_pot(&self, id: &Id) -> Result<Id, Self::Error> {
        self.core_storage().rewards_pot(id).map_err(ApiError::from)
    }
}

impl<'a, Store> MutableDappStore for Api<'a, Hub, Store>
where
    Store: MutStorage,
{
    fn add_dapp(&mut self, id: &Id, name: String) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .add_dapp(id, name)
            .map_err(ApiError::from)
    }

    fn remove_dapp(&mut self, id: &Id) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .remove_dapp(id)
            .map_err(ApiError::from)
    }

    fn set_percent(&mut self, id: &Id, percent: NonZeroPercent) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .set_percent(id, percent)
            .map_err(ApiError::from)
    }

    fn set_collector(&mut self, id: &Id, collector: Id) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .set_collector(id, collector)
            .map_err(ApiError::from)
    }

    fn set_repo_url(&mut self, id: &Id, repo_url: String) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .set_repo_url(id, repo_url)
            .map_err(ApiError::from)
    }

    fn set_rewards_pot(&mut self, id: &Id, rewards_pot: Id) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .set_rewards_pot(id, rewards_pot)
            .map_err(ApiError::from)
    }
}

impl<'a, Store> ReadonlyReferralStore for Api<'a, Hub, Store>
where
    Store: Storage,
{
    fn code_exists(&self, code: ReferralCode) -> Result<bool, Self::Error> {
        self.core_storage()
            .code_exists(code)
            .map_err(ApiError::from)
    }

    fn owner_exists(&self, owner: &Id) -> Result<bool, Self::Error> {
        self.core_storage()
            .owner_exists(owner)
            .map_err(ApiError::from)
    }

    fn owner_of(&self, code: ReferralCode) -> Result<Option<Id>, Self::Error> {
        self.core_storage().owner_of(code).map_err(ApiError::from)
    }

    fn latest(&self) -> Result<Option<ReferralCode>, Self::Error> {
        self.core_storage().latest().map_err(ApiError::from)
    }

    fn total_earnings(&self, code: ReferralCode) -> Result<Option<NonZeroU128>, Self::Error> {
        self.core_storage()
            .total_earnings(code)
            .map_err(ApiError::from)
    }

    fn dapp_earnings(
        &self,
        dapp: &Id,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        self.core_storage()
            .dapp_earnings(dapp, code)
            .map_err(ApiError::from)
    }

    fn dapp_contributions(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        self.core_storage()
            .dapp_contributions(dapp)
            .map_err(ApiError::from)
    }
}

impl<'a, Store> MutableReferralStore for Api<'a, Hub, Store>
where
    Store: MutStorage,
{
    fn set_latest(&mut self, code: ReferralCode) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .set_latest(code)
            .map_err(ApiError::from)
    }

    fn set_code_owner(&mut self, code: ReferralCode, owner: Id) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .set_code_owner(code, owner)
            .map_err(ApiError::from)
    }

    fn increment_invocations(&mut self, dapp: &Id, code: ReferralCode) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .increment_invocations(dapp, code)
            .map_err(ApiError::from)
    }

    fn set_total_earnings(
        &mut self,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .set_total_earnings(code, total)
            .map_err(ApiError::from)
    }

    fn set_dapp_earnings(
        &mut self,
        dapp: &Id,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .set_dapp_earnings(dapp, code, total)
            .map_err(ApiError::from)
    }

    fn set_dapp_contributions(
        &mut self,
        dapp: &Id,
        contributions: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .set_dapp_contributions(dapp, contributions)
            .map_err(ApiError::from)
    }
}

impl<'a, Store> ReadonlyCollectStore for Api<'a, Hub, Store>
where
    Store: Storage,
{
    fn referrer_total_collected(
        &self,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        self.core_storage()
            .referrer_total_collected(code)
            .map_err(ApiError::from)
    }

    fn referrer_dapp_collected(
        &self,
        dapp: &Id,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        self.core_storage()
            .referrer_dapp_collected(dapp, code)
            .map_err(ApiError::from)
    }

    fn dapp_total_collected(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        self.core_storage()
            .dapp_total_collected(dapp)
            .map_err(ApiError::from)
    }
}

impl<'a, Store> MutableCollectStore for Api<'a, Hub, Store>
where
    Store: MutStorage,
{
    fn set_referrer_total_collected(
        &mut self,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .set_referrer_total_collected(code, total)
            .map_err(ApiError::from)
    }

    fn set_referrer_dapp_collected(
        &mut self,
        dapp: &Id,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .set_referrer_dapp_collected(dapp, code, total)
            .map_err(ApiError::from)
    }

    fn set_dapp_total_collected(
        &mut self,
        dapp: &Id,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.core_storage_mut()
            .set_dapp_total_collected(dapp, total)
            .map_err(ApiError::from)
    }
}
