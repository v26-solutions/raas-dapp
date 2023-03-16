use std::num::NonZeroU128;

use cosmwasm_std::{Env, Querier, QuerierWrapper, StdError, Storage as CwStorage};

use archway_bindings::{types::rewards::ContractMetadataResponse, ArchwayQuery};

use referrals_core::{
    CollectQuery, CollectStore, DappQuery, DappStore, FallibleApi, Id, NonZeroPercent,
    ReferralCode, ReferralStore,
};
use referrals_cw::rewards_pot::{AdminResponse, QueryMsg as RewardsPotQuery, TotalRewardsResponse};
use referrals_storage::{Error as CoreStorageError, Storage as CoreStorage};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Store(#[from] CoreStorageError<crate::StoreError>),
    #[error(transparent)]
    Query(#[from] StdError),
}

pub(crate) struct CoreDeps<'a> {
    storage: CoreStorage<crate::MutStore<'a>>,
    env: &'a Env,
    querier: QuerierWrapper<'a, ArchwayQuery>,
}

impl<'a> CoreDeps<'a> {
    pub fn new(storage: &'a mut dyn CwStorage, env: &'a Env, querier: &'a dyn Querier) -> Self {
        Self {
            storage: CoreStorage::new(crate::MutStore::from_repo(storage)),
            env,
            querier: QuerierWrapper::new(querier),
        }
    }
}

impl<'a> FallibleApi for CoreDeps<'a> {
    type Error = Error;
}

impl<'a> DappQuery for CoreDeps<'a> {
    fn self_id(&self) -> Result<Id, Self::Error> {
        Ok(Id::from(self.env.contract.address.clone()))
    }

    fn rewards_admin(&self, id: &Id) -> Result<Id, Self::Error> {
        let contract_metadata: ContractMetadataResponse = self
            .querier
            .query(&ArchwayQuery::contract_metadata(id.clone().into_string()).into())?;
        Ok(Id::from(contract_metadata.owner_address))
    }

    fn rewards_pot_admin(&self, id: &Id) -> Result<Id, Self::Error> {
        let response: AdminResponse = self
            .querier
            .query_wasm_smart(id.clone().into_string(), &RewardsPotQuery::Admin {})?;
        Ok(Id::from(response.admin))
    }

    fn current_fee(&self, id: &Id) -> Result<NonZeroU128, Self::Error> {
        todo!()
    }
}

impl<'a> CollectQuery for CoreDeps<'a> {
    fn dapp_total_rewards(&self, pot: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        let response: TotalRewardsResponse = self
            .querier
            .query_wasm_smart(pot.clone().into_string(), &RewardsPotQuery::TotalRewards {})?;
        Ok(NonZeroU128::new(response.total.u128()))
    }
}

// Delegation to CoreStorage boilerplate

impl<'a> DappStore for CoreDeps<'a> {
    fn dapp_exists(&self, id: &Id) -> Result<bool, Self::Error> {
        self.storage.dapp_exists(id).map_err(Error::from)
    }

    fn remove_dapp(&mut self, id: &Id) -> Result<(), Self::Error> {
        self.storage.remove_dapp(id).map_err(Error::from)
    }

    fn set_percent(&mut self, id: &Id, percent: NonZeroPercent) -> Result<(), Self::Error> {
        self.storage.set_percent(id, percent).map_err(Error::from)
    }

    fn percent(&self, id: &Id) -> Result<NonZeroPercent, Self::Error> {
        self.storage.percent(id).map_err(Error::from)
    }

    fn set_collector(&mut self, id: &Id, collector: Id) -> Result<(), Self::Error> {
        self.storage
            .set_collector(id, collector)
            .map_err(Error::from)
    }

    fn collector(&self, id: &Id) -> Result<Id, Self::Error> {
        self.storage.collector(id).map_err(Error::from)
    }

    fn set_repo_url(&mut self, id: &Id, repo_url: String) -> Result<(), Self::Error> {
        self.storage.set_repo_url(id, repo_url).map_err(Error::from)
    }

    fn set_rewards_pot(&mut self, id: &Id, rewards_pot: Id) -> Result<(), Self::Error> {
        self.storage
            .set_rewards_pot(id, rewards_pot)
            .map_err(Error::from)
    }

    fn has_rewards_pot(&mut self, id: &Id) -> Result<bool, Self::Error> {
        self.storage.has_rewards_pot(id).map_err(Error::from)
    }

    fn rewards_pot(&self, id: &Id) -> Result<Id, Self::Error> {
        self.storage.rewards_pot(id).map_err(Error::from)
    }
}

impl<'a> ReferralStore for CoreDeps<'a> {
    fn code_exists(&self, code: ReferralCode) -> Result<bool, Self::Error> {
        self.storage.code_exists(code).map_err(Error::from)
    }

    fn owner_exists(&self, owner: &Id) -> Result<bool, Self::Error> {
        self.storage.owner_exists(owner).map_err(Error::from)
    }

    fn owner_of(&self, code: ReferralCode) -> Result<Option<Id>, Self::Error> {
        self.storage.owner_of(code).map_err(Error::from)
    }

    fn set_latest(&mut self, code: ReferralCode) -> Result<(), Self::Error> {
        self.storage.set_latest(code).map_err(Error::from)
    }

    fn latest(&self) -> Result<ReferralCode, Self::Error> {
        self.storage.latest().map_err(Error::from)
    }

    fn set_code_owner(&mut self, code: ReferralCode, owner: Id) -> Result<(), Self::Error> {
        self.storage
            .set_code_owner(code, owner)
            .map_err(Error::from)
    }

    fn increment_invocations(&mut self, dapp: &Id, code: ReferralCode) -> Result<(), Self::Error> {
        self.storage
            .increment_invocations(dapp, code)
            .map_err(Error::from)
    }

    fn set_total_earnings(
        &mut self,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.storage
            .set_total_earnings(code, total)
            .map_err(Error::from)
    }

    fn total_earnings(&self, code: ReferralCode) -> Result<Option<NonZeroU128>, Self::Error> {
        self.storage.total_earnings(code).map_err(Error::from)
    }

    fn set_dapp_earnings(
        &mut self,
        dapp: &Id,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.storage
            .set_dapp_earnings(dapp, code, total)
            .map_err(Error::from)
    }

    fn dapp_earnings(
        &self,
        dapp: &Id,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        self.storage.dapp_earnings(dapp, code).map_err(Error::from)
    }

    fn set_dapp_contributions(
        &mut self,
        dapp: &Id,
        contributions: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.storage
            .set_dapp_contributions(dapp, contributions)
            .map_err(Error::from)
    }

    fn dapp_contributions(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        self.storage.dapp_contributions(dapp).map_err(Error::from)
    }
}

impl<'a> CollectStore for CoreDeps<'a> {
    fn set_referrer_total_collected(
        &mut self,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.storage
            .set_referrer_total_collected(code, total)
            .map_err(Error::from)
    }

    fn referrer_total_collected(
        &self,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        self.storage
            .referrer_total_collected(code)
            .map_err(Error::from)
    }

    fn set_referrer_dapp_collected(
        &mut self,
        dapp: &Id,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.storage
            .set_referrer_dapp_collected(dapp, code, total)
            .map_err(Error::from)
    }

    fn referrer_dapp_collected(
        &self,
        dapp: &Id,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        self.storage
            .referrer_dapp_collected(dapp, code)
            .map_err(Error::from)
    }

    fn set_dapp_total_collected(
        &mut self,
        dapp: &Id,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.storage
            .set_dapp_total_collected(dapp, total)
            .map_err(Error::from)
    }

    fn dapp_total_collected(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        self.storage.dapp_total_collected(dapp).map_err(Error::from)
    }
}
