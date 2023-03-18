#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use std::num::NonZeroU128;

use referrals_core::{
    FallibleApi, Id, MutableCollectStore, MutableDappStore, MutableReferralStore, NonZeroPercent,
    ReadonlyCollectStore, ReadonlyDappStore, ReadonlyReferralStore, ReferralCode,
};

use kv_storage::{MutStorage as MutKvStorage, Storage as ReadonlyKvStorage};

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

mod dapp {
    use ::kv_storage::{map, Map};

    pub static DAPPS: Map<1024, &str, String> = map!("dapps");

    pub static PERCENT: Map<1024, &str, u8> = map!("percent");

    pub static COLLECTOR: Map<1024, &str, String> = map!("collector");

    pub static REPO_URL: Map<1024, &str, String> = map!("repo_url");

    pub static REWARDS_POT: Map<1024, &str, String> = map!("rewards_pot");
}

impl<T> ReadonlyDappStore for Storage<T>
where
    T: ReadonlyKvStorage,
{
    fn dapp_exists(&self, id: &Id) -> Result<bool, Self::Error> {
        dapp::DAPPS
            .has_key(&self.0, &id.as_ref())
            .map_err(Error::from)
    }

    fn percent(&self, id: &Id) -> Result<NonZeroPercent, Self::Error> {
        dapp::PERCENT
            .may_load(&self.0, &id.as_ref())?
            .ok_or(Error::NotFound)
            .map(NonZeroPercent::new)
            .map(Option::unwrap) // safe as only NonZeroPercent's accepted into storage
    }

    fn collector(&self, id: &Id) -> Result<Id, Self::Error> {
        dapp::COLLECTOR
            .may_load(&self.0, &id.as_ref())?
            .ok_or(Error::NotFound)
            .map(Id::from)
    }

    fn has_rewards_pot(&self, id: &Id) -> Result<bool, Self::Error> {
        dapp::REWARDS_POT
            .has_key(&self.0, &id.as_ref())
            .map_err(Error::from)
    }

    fn rewards_pot(&self, id: &Id) -> Result<Id, Self::Error> {
        dapp::REWARDS_POT
            .may_load(&self.0, &id.as_ref())?
            .ok_or(Error::NotFound)
            .map(Id::from)
    }
}

impl<T> MutableDappStore for Storage<T>
where
    T: MutKvStorage,
{
    fn add_dapp(&mut self, id: &Id, name: String) -> Result<(), Self::Error> {
        dapp::DAPPS
            .save(&mut self.0, &id.as_ref(), &name)
            .map_err(Error::from)
    }

    fn remove_dapp(&mut self, id: &Id) -> Result<(), Self::Error> {
        dapp::DAPPS
            .remove(&mut self.0, &id.as_ref())
            .map_err(Error::from)
    }

    fn set_percent(&mut self, id: &Id, percent: NonZeroPercent) -> Result<(), Self::Error> {
        dapp::PERCENT
            .save(&mut self.0, &id.as_ref(), &percent.to_u8())
            .map_err(Error::from)
    }

    fn set_collector(&mut self, id: &Id, collector: Id) -> Result<(), Self::Error> {
        dapp::COLLECTOR
            .save(&mut self.0, &id.as_ref(), collector.as_ref())
            .map_err(Error::from)
    }

    fn set_repo_url(&mut self, id: &Id, repo_url: String) -> Result<(), Self::Error> {
        dapp::REPO_URL
            .save(&mut self.0, &id.as_ref(), &repo_url)
            .map_err(Error::from)
    }

    fn set_rewards_pot(&mut self, id: &Id, rewards_pot: Id) -> Result<(), Self::Error> {
        dapp::REWARDS_POT
            .save(&mut self.0, &id.as_ref(), rewards_pot.as_ref())
            .map_err(Error::from)
    }
}

mod referral {
    use std::num::NonZeroU128;

    use kv_storage::{item, map, Item, Map};

    pub static CODES: Map<1024, u64, String> = map!("codes");

    pub static CODE_OWNERS: Map<1024, &str, u64> = map!("code_owners");

    pub static LATEST_CODE: Item<u64> = item!("latest_code");

    pub static INVOCATION_COUNTS: Map<1024, (&str, u64), u64> = map!("invocation_counts");

    pub static CODE_TOTAL_EARNINGS: Map<1024, u64, NonZeroU128> = map!("code_total_earnings");

    pub static CODE_DAPP_EARNINGS: Map<1024, (&str, u64), NonZeroU128> = map!("code_dapp_earnings");

    pub static DAPP_CONTRIBUTIONS: Map<1024, &str, NonZeroU128> = map!("dapp_contributions");
}

impl<T> ReadonlyReferralStore for Storage<T>
where
    T: ReadonlyKvStorage,
{
    fn code_exists(&self, code: ReferralCode) -> Result<bool, Self::Error> {
        referral::CODES
            .has_key(&self.0, &code.to_u64())
            .map_err(Error::from)
    }

    fn owner_exists(&self, owner: &Id) -> Result<bool, Self::Error> {
        referral::CODE_OWNERS
            .has_key(&self.0, &owner.as_ref())
            .map_err(Error::from)
    }

    fn owner_of(&self, code: ReferralCode) -> Result<Option<Id>, Self::Error> {
        referral::CODES
            .may_load(&self.0, &code.to_u64())
            .map(|maybe| maybe.map(Id::from))
            .map_err(Error::from)
    }

    fn latest(&self) -> Result<Option<ReferralCode>, Self::Error> {
        referral::LATEST_CODE
            .may_load(&self.0)
            .map(|maybe_code| maybe_code.map(ReferralCode::from))
            .map_err(Error::from)
    }

    fn total_earnings(&self, code: ReferralCode) -> Result<Option<NonZeroU128>, Self::Error> {
        referral::CODE_TOTAL_EARNINGS
            .may_load(&self.0, &code.to_u64())
            .map_err(Error::from)
    }

    fn dapp_earnings(
        &self,
        dapp: &Id,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        referral::CODE_DAPP_EARNINGS
            .may_load(&self.0, &(dapp.as_ref(), code.to_u64()))
            .map_err(Error::from)
    }

    fn dapp_contributions(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        referral::DAPP_CONTRIBUTIONS
            .may_load(&self.0, &dapp.as_ref())
            .map_err(Error::from)
    }
}

impl<T> MutableReferralStore for Storage<T>
where
    T: MutKvStorage,
{
    fn set_latest(&mut self, code: ReferralCode) -> Result<(), Self::Error> {
        referral::LATEST_CODE
            .save(&mut self.0, &code.to_u64())
            .map_err(Error::from)
    }

    fn set_code_owner(&mut self, code: ReferralCode, owner: Id) -> Result<(), Self::Error> {
        referral::CODES.save(&mut self.0, &code.to_u64(), owner.as_ref())?;
        referral::CODE_OWNERS.save(&mut self.0, &owner.as_ref(), &code.to_u64())?;
        Ok(())
    }

    fn increment_invocations(&mut self, dapp: &Id, code: ReferralCode) -> Result<(), Self::Error> {
        let current = referral::INVOCATION_COUNTS
            .may_load(&self.0, &(dapp.as_ref(), code.to_u64()))?
            .unwrap_or_default();

        referral::INVOCATION_COUNTS
            .save(&mut self.0, &(dapp.as_ref(), code.to_u64()), &(current + 1))
            .map_err(Error::from)
    }

    fn set_total_earnings(
        &mut self,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        referral::CODE_TOTAL_EARNINGS
            .save(&mut self.0, &code.to_u64(), &total)
            .map_err(Error::from)
    }

    fn set_dapp_earnings(
        &mut self,
        dapp: &Id,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        referral::CODE_DAPP_EARNINGS
            .save(&mut self.0, &(dapp.as_ref(), code.to_u64()), &total)
            .map_err(Error::from)
    }

    fn set_dapp_contributions(
        &mut self,
        dapp: &Id,
        contributions: NonZeroU128,
    ) -> Result<(), Self::Error> {
        referral::DAPP_CONTRIBUTIONS
            .save(&mut self.0, &dapp.as_ref(), &contributions)
            .map_err(Error::from)
    }
}

mod collect {
    use std::num::NonZeroU128;

    use kv_storage::{map, Map};

    pub static REFERRER_TOTAL: Map<1024, u64, NonZeroU128> = map!("referrer_total");

    pub static REFERRER_DAPP: Map<1024, (&str, u64), NonZeroU128> = map!("referrer_dapp");

    pub static DAPP_TOTAL: Map<1024, &str, NonZeroU128> = map!("dapp_total");
}

impl<T> ReadonlyCollectStore for Storage<T>
where
    T: ReadonlyKvStorage,
{
    fn referrer_total_collected(
        &self,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        collect::REFERRER_TOTAL
            .may_load(&self.0, &code.to_u64())
            .map_err(Error::from)
    }

    fn referrer_dapp_collected(
        &self,
        dapp: &Id,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        collect::REFERRER_DAPP
            .may_load(&self.0, &(dapp.as_ref(), code.to_u64()))
            .map_err(Error::from)
    }

    fn dapp_total_collected(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        collect::DAPP_TOTAL
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
        collect::REFERRER_TOTAL
            .save(&mut self.0, &code.to_u64(), &total)
            .map_err(Error::from)
    }

    fn set_referrer_dapp_collected(
        &mut self,
        dapp: &Id,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        collect::REFERRER_DAPP
            .save(&mut self.0, &(dapp.as_ref(), code.to_u64()), &total)
            .map_err(Error::from)
    }

    fn set_dapp_total_collected(
        &mut self,
        dapp: &Id,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        collect::DAPP_TOTAL
            .save(&mut self.0, &dapp.as_ref(), &total)
            .map_err(Error::from)
    }
}
