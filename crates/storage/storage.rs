#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use referrals_core::FallibleApi;

use kv_storage::Storage as ReadonlyKvStorage;

#[derive(Debug, thiserror::Error)]
pub enum Error<S> {
    #[error(transparent)]
    Storage(#[from] S),
    #[error("not found")]
    NotFound,
    #[error("index out of bounds")]
    IndexOutOfBounds,
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

mod hub {
    use std::num::NonZeroU128;

    use referrals_core::hub::{
        DappsQuery, MutableCollectStore, MutableDappStore, MutableReferralStore, NonZeroPercent,
        ReadonlyCollectStore, ReadonlyDappStore, ReadonlyReferralStore, ReferralCode,
    };
    use referrals_core::Id;

    use kv_storage::{MutStorage as MutKvStorage, Storage as ReadonlyKvStorage};

    use crate::{Error, Storage};

    mod dapp {
        use ::kv_storage::{item, map, Item, Map};

        pub static DAPP_LAST_INDEX: Item<u64> = item!("dapp_last_index");

        pub static DAPP_INDEX: Map<1024, u64, String> = map!("dapp_index");

        pub static DAPP_REVERSE_INDEX: Map<1024, &str, u64> = map!("dapp_reverse_index");

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
                .has_key(&self.0, id.as_str())
                .map_err(Error::from)
        }

        fn percent(&self, id: &Id) -> Result<NonZeroPercent, Self::Error> {
            dapp::PERCENT
                .may_load(&self.0, id.as_str())?
                .ok_or(Error::NotFound)
                .map(NonZeroPercent::new)
                .map(Option::unwrap) // safe as only NonZeroPercent's accepted into storage
        }

        fn collector(&self, id: &Id) -> Result<Id, Self::Error> {
            dapp::COLLECTOR
                .may_load(&self.0, id.as_str())?
                .ok_or(Error::NotFound)
                .map(Id::from)
        }

        fn has_rewards_pot(&self, id: &Id) -> Result<bool, Self::Error> {
            dapp::REWARDS_POT
                .has_key(&self.0, id.as_str())
                .map_err(Error::from)
        }

        fn rewards_pot(&self, id: &Id) -> Result<Id, Self::Error> {
            dapp::REWARDS_POT
                .may_load(&self.0, id.as_str())?
                .ok_or(Error::NotFound)
                .map(Id::from)
        }
    }

    impl<T> MutableDappStore for Storage<T>
    where
        T: MutKvStorage,
    {
        fn add_dapp(&mut self, id: &Id, name: String) -> Result<(), Self::Error> {
            if !dapp::DAPP_REVERSE_INDEX.has_key(&self.0, id.as_str())? {
                let index = dapp::DAPP_LAST_INDEX
                    .may_load(&self.0)?
                    .map_or(0, |count| count + 1);

                dapp::DAPP_INDEX.save(&mut self.0, index, id.as_str().to_owned())?;
                dapp::DAPP_REVERSE_INDEX.save(&mut self.0, id.as_str(), index)?;
                dapp::DAPP_LAST_INDEX.save(&mut self.0, index)?;
            }

            dapp::DAPPS
                .save(&mut self.0, id.as_str(), &name)
                .map_err(Error::from)
        }

        fn remove_dapp(&mut self, id: &Id) -> Result<(), Self::Error> {
            dapp::DAPPS
                .remove(&mut self.0, id.as_str())
                .map_err(Error::from)
        }

        fn set_percent(&mut self, id: &Id, percent: NonZeroPercent) -> Result<(), Self::Error> {
            dapp::PERCENT
                .save(&mut self.0, id.as_str(), percent.to_u8())
                .map_err(Error::from)
        }

        fn set_collector(&mut self, id: &Id, collector: Id) -> Result<(), Self::Error> {
            dapp::COLLECTOR
                .save(&mut self.0, id.as_str(), collector.as_ref())
                .map_err(Error::from)
        }

        fn set_repo_url(&mut self, id: &Id, repo_url: String) -> Result<(), Self::Error> {
            dapp::REPO_URL
                .save(&mut self.0, id.as_str(), repo_url)
                .map_err(Error::from)
        }

        fn set_rewards_pot(&mut self, id: &Id, rewards_pot: Id) -> Result<(), Self::Error> {
            dapp::REWARDS_POT
                .save(&mut self.0, id.as_str(), rewards_pot.as_ref())
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

        pub static TOTAL_INVOCATION_COUNTS: Map<1024, &str, u64> = map!("total_invocation_counts");

        pub static DISCRETE_REFERRERS: Map<1024, &str, u64> = map!("discrete_referrers");

        pub static CODE_TOTAL_EARNINGS: Map<1024, u64, NonZeroU128> = map!("code_total_earnings");

        pub static CODE_DAPP_EARNINGS: Map<1024, (&str, u64), NonZeroU128> =
            map!("code_dapp_earnings");

        pub static DAPP_CONTRIBUTIONS: Map<1024, &str, NonZeroU128> = map!("dapp_contributions");
    }

    impl<T> ReadonlyReferralStore for Storage<T>
    where
        T: ReadonlyKvStorage,
    {
        fn code_exists(&self, code: ReferralCode) -> Result<bool, Self::Error> {
            referral::CODES
                .has_key(&self.0, code.to_u64())
                .map_err(Error::from)
        }

        fn owner_exists(&self, owner: &Id) -> Result<bool, Self::Error> {
            referral::CODE_OWNERS
                .has_key(&self.0, owner.as_str())
                .map_err(Error::from)
        }

        fn owner_of(&self, code: ReferralCode) -> Result<Option<Id>, Self::Error> {
            referral::CODES
                .may_load(&self.0, code.to_u64())
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
                .may_load(&self.0, code.to_u64())
                .map_err(Error::from)
        }

        fn dapp_earnings(
            &self,
            dapp: &Id,
            code: ReferralCode,
        ) -> Result<Option<NonZeroU128>, Self::Error> {
            referral::CODE_DAPP_EARNINGS
                .may_load(&self.0, (dapp.as_str(), code.to_u64()))
                .map_err(Error::from)
        }

        fn dapp_contributions(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
            referral::DAPP_CONTRIBUTIONS
                .may_load(&self.0, dapp.as_str())
                .map_err(Error::from)
        }
    }

    impl<T> MutableReferralStore for Storage<T>
    where
        T: MutKvStorage,
    {
        fn set_latest(&mut self, code: ReferralCode) -> Result<(), Self::Error> {
            referral::LATEST_CODE
                .save(&mut self.0, code.to_u64())
                .map_err(Error::from)
        }

        fn set_code_owner(&mut self, code: ReferralCode, owner: Id) -> Result<(), Self::Error> {
            referral::CODES.save(&mut self.0, code.to_u64(), owner.as_ref())?;
            referral::CODE_OWNERS.save(&mut self.0, owner.as_str(), code.to_u64())?;
            Ok(())
        }

        fn increment_invocations(
            &mut self,
            dapp: &Id,
            code: ReferralCode,
        ) -> Result<(), Self::Error> {
            let current_per_referrer = referral::INVOCATION_COUNTS
                .may_load(&self.0, (dapp.as_str(), code.to_u64()))?
                .unwrap_or(0);

            if current_per_referrer == 0 {
                let discrete_referrers = referral::DISCRETE_REFERRERS
                    .may_load(&self.0, dapp.as_str())?
                    .unwrap_or(0);

                referral::DISCRETE_REFERRERS.save(
                    &mut self.0,
                    dapp.as_str(),
                    discrete_referrers + 1,
                )?;
            }

            let current_total = referral::TOTAL_INVOCATION_COUNTS
                .may_load(&self.0, dapp.as_str())?
                .unwrap_or_default();

            referral::INVOCATION_COUNTS.save(
                &mut self.0,
                (dapp.as_str(), code.to_u64()),
                current_per_referrer + 1,
            )?;

            referral::TOTAL_INVOCATION_COUNTS
                .save(&mut self.0, dapp.as_str(), current_total + 1)
                .map_err(Error::from)
        }

        fn set_total_earnings(
            &mut self,
            code: ReferralCode,
            total: NonZeroU128,
        ) -> Result<(), Self::Error> {
            referral::CODE_TOTAL_EARNINGS
                .save(&mut self.0, code.to_u64(), total)
                .map_err(Error::from)
        }

        fn set_dapp_earnings(
            &mut self,
            dapp: &Id,
            code: ReferralCode,
            total: NonZeroU128,
        ) -> Result<(), Self::Error> {
            referral::CODE_DAPP_EARNINGS
                .save(&mut self.0, (dapp.as_str(), code.to_u64()), total)
                .map_err(Error::from)
        }

        fn set_dapp_contributions(
            &mut self,
            dapp: &Id,
            contributions: NonZeroU128,
        ) -> Result<(), Self::Error> {
            referral::DAPP_CONTRIBUTIONS
                .save(&mut self.0, dapp.as_str(), contributions)
                .map_err(Error::from)
        }
    }

    // implementation requires stores from both `dapp` & `referral`
    impl<T> DappsQuery for Storage<T>
    where
        T: ReadonlyKvStorage,
    {
        fn total_dapp_count(&self) -> Result<u64, Self::Error> {
            dapp::DAPP_LAST_INDEX
                .may_load(&self.0)
                // add 1 to 0-based index
                .map(|maybe_idx| maybe_idx.map_or(0, |idx| idx + 1))
                .map_err(Error::from)
        }

        fn all_dapp_ids(
            &self,
            start: Option<u64>,
            limit: Option<u64>,
        ) -> Result<Vec<Id>, Self::Error> {
            let Some(last_index) = dapp::DAPP_LAST_INDEX.may_load(&self.0)? else {
                return Ok(vec![]);
            };

            let start = start.unwrap_or(0);

            if start > last_index {
                return Err(Error::IndexOutOfBounds);
            }

            let limit = match limit {
                Some(l) => (start + l).min(last_index),
                None => last_index,
            };

            (start..=limit)
                .map(|idx| {
                    dapp::DAPP_INDEX
                        .may_load(&self.0, idx)
                        .map_err(Error::from)?
                        .ok_or(Error::NotFound)
                        .map(Id::from)
                })
                .collect()
        }

        fn dapp_name(&self, dapp: &Id) -> Result<Option<String>, Self::Error> {
            dapp::DAPPS
                .may_load(&self.0, dapp.as_str())
                .map_err(Error::from)
        }

        fn dapp_repo_url(&self, dapp: &Id) -> Result<Option<String>, Self::Error> {
            dapp::REPO_URL
                .may_load(&self.0, dapp.as_str())
                .map_err(Error::from)
        }

        fn dapp_total_invocations(&self, dapp: &Id) -> Result<u64, Self::Error> {
            referral::TOTAL_INVOCATION_COUNTS
                .may_load(&self.0, dapp.as_str())
                .map(|maybe_count| maybe_count.unwrap_or(0))
                .map_err(Error::from)
        }

        fn dapp_discrete_referrers(&self, dapp: &Id) -> Result<u64, Self::Error> {
            referral::DISCRETE_REFERRERS
                .may_load(&self.0, dapp.as_str())
                .map(|maybe_count| maybe_count.unwrap_or(0))
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
                .may_load(&self.0, code.to_u64())
                .map_err(Error::from)
        }

        fn referrer_dapp_collected(
            &self,
            dapp: &Id,
            code: ReferralCode,
        ) -> Result<Option<NonZeroU128>, Self::Error> {
            collect::REFERRER_DAPP
                .may_load(&self.0, (dapp.as_str(), code.to_u64()))
                .map_err(Error::from)
        }

        fn dapp_total_collected(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
            collect::DAPP_TOTAL
                .may_load(&self.0, dapp.as_str())
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
                .save(&mut self.0, code.to_u64(), total)
                .map_err(Error::from)
        }

        fn set_referrer_dapp_collected(
            &mut self,
            dapp: &Id,
            code: ReferralCode,
            total: NonZeroU128,
        ) -> Result<(), Self::Error> {
            collect::REFERRER_DAPP
                .save(&mut self.0, (dapp.as_str(), code.to_u64()), total)
                .map_err(Error::from)
        }

        fn set_dapp_total_collected(
            &mut self,
            dapp: &Id,
            total: NonZeroU128,
        ) -> Result<(), Self::Error> {
            collect::DAPP_TOTAL
                .save(&mut self.0, dapp.as_str(), total)
                .map_err(Error::from)
        }
    }
}
