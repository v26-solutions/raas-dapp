#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use std::num::NonZeroU128;

use referrals_core::{
    FallibleApi, Id, MutableCollectStore, MutableDappStore, MutableReferralStore, NonZeroPercent,
    ReadonlyCollectStore, ReadonlyDappStore, ReadonlyReferralStore, ReferralCode,
};

use kv_storage::{item, map, Item, Map, MutStorage as MutKvStorage, Storage as ReadonlyKvStorage};

#[derive(Debug, thiserror::Error)]
pub enum Error<S> {
    #[error(transparent)]
    Storage(#[from] S),
    #[error("not found")]
    NotFound,
}

pub struct Storage<T>(T);

impl<T> Storage<T> {
    pub fn new(storage: T) -> Self {
        Self(storage)
    }

    pub fn inner(&self) -> &T {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> FallibleApi for Storage<T>
where
    T: ReadonlyKvStorage,
{
    type Error = Error<T::Error>;
}

static DAPPS: Map<1024, &str, String> = map!("dapps");

static DAPP_PERCENT: Map<1024, &str, u8> = map!("dapp_percent");

static DAPP_COLLECTOR: Map<1024, &str, String> = map!("dapp_collector");

static DAPP_REPO_URL: Map<1024, &str, String> = map!("dapp_repo_url");

static DAPP_REWARDS_POT: Map<1024, &str, String> = map!("dapp_rewards_pot");

impl<T> ReadonlyDappStore for Storage<T>
where
    T: ReadonlyKvStorage,
{
    fn dapp_exists(&self, id: &Id) -> Result<bool, Self::Error> {
        DAPPS.has_key(&self.0, &id.as_ref()).map_err(Error::from)
    }

    fn percent(&self, id: &Id) -> Result<NonZeroPercent, Self::Error> {
        DAPP_PERCENT
            .may_load(&self.0, &id.as_ref())?
            .ok_or(Error::NotFound)
            .map(NonZeroPercent::new)
            .map(Option::unwrap) // safe as only NonZeroPercent's accepted into storage
    }

    fn collector(&self, id: &Id) -> Result<Id, Self::Error> {
        DAPP_COLLECTOR
            .may_load(&self.0, &id.as_ref())?
            .ok_or(Error::NotFound)
            .map(Id::from)
    }

    fn has_rewards_pot(&self, id: &Id) -> Result<bool, Self::Error> {
        DAPP_REWARDS_POT
            .has_key(&self.0, &id.as_ref())
            .map_err(Error::from)
    }

    fn rewards_pot(&self, id: &Id) -> Result<Id, Self::Error> {
        DAPP_COLLECTOR
            .may_load(&self.0, &id.as_ref())?
            .ok_or(Error::NotFound)
            .map(Id::from)
    }
}

impl<T> MutableDappStore for Storage<T>
where
    T: MutKvStorage,
{
    fn remove_dapp(&mut self, id: &Id) -> Result<(), Self::Error> {
        DAPPS.remove(&mut self.0, &id.as_ref()).map_err(Error::from)
    }

    fn set_percent(&mut self, id: &Id, percent: NonZeroPercent) -> Result<(), Self::Error> {
        DAPP_PERCENT
            .save(&mut self.0, &id.as_ref(), &percent.to_u8())
            .map_err(Error::from)
    }

    fn set_collector(&mut self, id: &Id, collector: Id) -> Result<(), Self::Error> {
        DAPP_COLLECTOR
            .save(&mut self.0, &id.as_ref(), collector.as_ref())
            .map_err(Error::from)
    }

    fn set_repo_url(&mut self, id: &Id, repo_url: String) -> Result<(), Self::Error> {
        DAPP_REPO_URL
            .save(&mut self.0, &id.as_ref(), &repo_url)
            .map_err(Error::from)
    }

    fn set_rewards_pot(&mut self, id: &Id, rewards_pot: Id) -> Result<(), Self::Error> {
        DAPP_REWARDS_POT
            .save(&mut self.0, &id.as_ref(), rewards_pot.as_ref())
            .map_err(Error::from)
    }
}

static CODES: Map<1024, u64, String> = map!("referral_codes");

static CODE_OWNERS: Map<1024, &str, u64> = map!("referral_code_owners");

static LATEST_CODE: Item<u64> = item!("latest_referral_code");

static INVOCATION_COUNTS: Map<1024, (&str, u64), u64> = map!("invocation_counts");

static CODE_TOTAL_EARNINGS: Map<1024, u64, NonZeroU128> = map!("code_total_earnings");

static CODE_DAPP_EARNINGS: Map<1024, (&str, u64), NonZeroU128> = map!("code_dapp_earnings");

static DAPP_CONTRIBUTIONS: Map<1024, &str, NonZeroU128> = map!("dapp_contributions");

impl<T> ReadonlyReferralStore for Storage<T>
where
    T: ReadonlyKvStorage,
{
    fn code_exists(&self, code: ReferralCode) -> Result<bool, Self::Error> {
        CODES.has_key(&self.0, &code.to_u64()).map_err(Error::from)
    }

    fn owner_exists(&self, owner: &Id) -> Result<bool, Self::Error> {
        CODE_OWNERS
            .has_key(&self.0, &owner.as_ref())
            .map_err(Error::from)
    }

    fn owner_of(&self, code: ReferralCode) -> Result<Option<Id>, Self::Error> {
        CODES
            .may_load(&self.0, &code.to_u64())
            .map(|maybe| maybe.map(Id::from))
            .map_err(Error::from)
    }

    fn latest(&self) -> Result<Option<ReferralCode>, Self::Error> {
        LATEST_CODE
            .may_load(&self.0)
            .map(|maybe_code| maybe_code.map(ReferralCode::from))
            .map_err(Error::from)
    }

    fn total_earnings(&self, code: ReferralCode) -> Result<Option<NonZeroU128>, Self::Error> {
        CODE_TOTAL_EARNINGS
            .may_load(&self.0, &code.to_u64())
            .map_err(Error::from)
    }

    fn dapp_earnings(
        &self,
        dapp: &Id,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        CODE_DAPP_EARNINGS
            .may_load(&self.0, &(dapp.as_ref(), code.to_u64()))
            .map_err(Error::from)
    }

    fn dapp_contributions(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        DAPP_CONTRIBUTIONS
            .may_load(&self.0, &dapp.as_ref())
            .map_err(Error::from)
    }
}

impl<T> MutableReferralStore for Storage<T>
where
    T: MutKvStorage,
{
    fn set_latest(&mut self, code: ReferralCode) -> Result<(), Self::Error> {
        LATEST_CODE
            .save(&mut self.0, &code.to_u64())
            .map_err(Error::from)
    }

    fn set_code_owner(&mut self, code: ReferralCode, owner: Id) -> Result<(), Self::Error> {
        CODES.save(&mut self.0, &code.to_u64(), owner.as_ref())?;
        CODE_OWNERS.save(&mut self.0, &owner.as_ref(), &code.to_u64())?;
        Ok(())
    }

    fn increment_invocations(&mut self, dapp: &Id, code: ReferralCode) -> Result<(), Self::Error> {
        let current = INVOCATION_COUNTS
            .may_load(&self.0, &(dapp.as_ref(), code.to_u64()))?
            .unwrap_or_default();

        INVOCATION_COUNTS
            .save(&mut self.0, &(dapp.as_ref(), code.to_u64()), &(current + 1))
            .map_err(Error::from)
    }

    fn set_total_earnings(
        &mut self,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        CODE_TOTAL_EARNINGS
            .save(&mut self.0, &code.to_u64(), &total)
            .map_err(Error::from)
    }

    fn set_dapp_earnings(
        &mut self,
        dapp: &Id,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        CODE_DAPP_EARNINGS
            .save(&mut self.0, &(dapp.as_ref(), code.to_u64()), &total)
            .map_err(Error::from)
    }

    fn set_dapp_contributions(
        &mut self,
        dapp: &Id,
        contributions: NonZeroU128,
    ) -> Result<(), Self::Error> {
        DAPP_CONTRIBUTIONS
            .save(&mut self.0, &dapp.as_ref(), &contributions)
            .map_err(Error::from)
    }
}

static REFERRER_TOTAL_COLLECTED: Map<1024, u64, NonZeroU128> = map!("referrer_total_collected");

static REFERRER_DAPP_COLLECTED: Map<1024, (&str, u64), NonZeroU128> =
    map!("referrer_dapp_collected");

static DAPP_TOTAL_COLLECTED: Map<1024, &str, NonZeroU128> = map!("dapp_total_collected");

impl<T> ReadonlyCollectStore for Storage<T>
where
    T: ReadonlyKvStorage,
{
    fn referrer_total_collected(
        &self,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        REFERRER_TOTAL_COLLECTED
            .may_load(&self.0, &code.to_u64())
            .map_err(Error::from)
    }

    fn referrer_dapp_collected(
        &self,
        dapp: &Id,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        REFERRER_DAPP_COLLECTED
            .may_load(&self.0, &(dapp.as_ref(), code.to_u64()))
            .map_err(Error::from)
    }

    fn dapp_total_collected(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        DAPP_TOTAL_COLLECTED
            .may_load(&self.0, &dapp.as_ref())
            .map_err(Error::from)
    }
}

impl<T> MutableCollectStore for Storage<T>
where
    T: MutKvStorage,
{
    fn set_referrer_total_collected(
        &mut self,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        REFERRER_TOTAL_COLLECTED
            .save(&mut self.0, &code.to_u64(), &total)
            .map_err(Error::from)
    }

    fn set_referrer_dapp_collected(
        &mut self,
        dapp: &Id,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        REFERRER_DAPP_COLLECTED
            .save(&mut self.0, &(dapp.as_ref(), code.to_u64()), &total)
            .map_err(Error::from)
    }

    fn set_dapp_total_collected(
        &mut self,
        dapp: &Id,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        DAPP_TOTAL_COLLECTED
            .save(&mut self.0, &dapp.as_ref(), &total)
            .map_err(Error::from)
    }
}
