use std::num::NonZeroU128;

use kv_storage::{item, map, Fallible, Item, Map, MutStorage, Storage};
use referrals_core::Id;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error<StoreError>(#[from] StoreError);

type StoreResult<Store, T = ()> = Result<T, Error<<Store as Fallible>::Error>>;

static REWARDS_DENOM: Item<String> = item!("rewards_denom");

/// Set the rewards denom
///
/// # Errors
///
/// This function will return an error if there is an issue with the underlying storage.
pub fn set_rewards_denom<Store: MutStorage>(
    store: &mut Store,
    denom: &String,
) -> StoreResult<Store> {
    REWARDS_DENOM.save(store, denom)?;
    Ok(())
}

/// Get the rewards denom
///
/// # Errors
///
/// This function will return an error if there is an issue with the underlying storage.
pub fn rewards_denom<Store: Storage>(store: &Store) -> StoreResult<Store, Option<String>> {
    REWARDS_DENOM.may_load(store).map_err(Error::from)
}

pub mod hub {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    static REWARD_POT_CODE_ID: Item<u64> = item!("reward_pot_code_id");
    static REWARD_POT_COUNT: Item<u64> = item!("reward_pot_count");
    static DAPP_FEES_CACHE: Map<1024, &str, NonZeroU128> = map!("dapp_fees_cache");

    /// Set the reward pot contract code id
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with the underlying storage.
    pub fn set_reward_pot_code_id<Store: MutStorage>(
        store: &mut Store,
        code_id: u64,
    ) -> StoreResult<Store> {
        REWARD_POT_CODE_ID.save(store, code_id)?;
        Ok(())
    }

    /// Get the reward pot contract code id
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with the underlying storage.
    pub fn reward_pot_code_id<Store: Storage>(store: &Store) -> StoreResult<Store, Option<u64>> {
        REWARD_POT_CODE_ID.may_load(store).map_err(Error::from)
    }

    /// Cache the dapp's flat fee
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with the underlying storage.
    pub fn cache_dapp_fee<Store: MutStorage>(
        store: &mut Store,
        dapp: &Id,
        fee: NonZeroU128,
    ) -> StoreResult<Store> {
        DAPP_FEES_CACHE.save(store, dapp.as_str(), fee)?;
        Ok(())
    }

    /// Get the dapp's cached flat fee
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with the underlying storage.
    pub fn cached_dapp_fee<Store: Storage>(
        store: &Store,
        dapp: &Id,
    ) -> StoreResult<Store, Option<NonZeroU128>> {
        DAPP_FEES_CACHE
            .may_load(store, dapp.as_str())
            .map_err(Error::from)
    }

    /// Increment the number of reward pots created, returning the new value.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with the underlying storage.
    pub fn increment_reward_pot_count<Store: MutStorage>(
        store: &mut Store,
    ) -> StoreResult<Store, u64> {
        let count = REWARD_POT_COUNT.may_load(store)?.unwrap_or_default();
        REWARD_POT_COUNT.save(store, count + 1)?;
        Ok(count)
    }
}

pub mod rewards_pot {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    static DAPP: Item<String> = item!("dapp");
    static CREATOR: Item<String> = item!("creator");
    static REWARDS_COLLECTED: Item<u128> = item!("rewards_collected");
    static REWARDS_RECORDS_COLLECTED: Item<u64> = item!("rewards_records_collected");

    /// Set owner dApp address
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an underlying storage issue.
    pub fn set_dapp<Store: MutStorage>(store: &mut Store, dapp: &String) -> StoreResult<Store, ()> {
        DAPP.save(store, dapp)?;
        Ok(())
    }

    /// Get owner dApp address
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an underlying storage issue.
    pub fn dapp<Store: Storage>(store: &Store) -> StoreResult<Store, Option<String>> {
        DAPP.may_load(store).map_err(Error::from)
    }

    /// Set creator address
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an underlying storage issue.
    pub fn set_creator<Store: MutStorage>(
        store: &mut Store,
        creator: &String,
    ) -> StoreResult<Store, ()> {
        CREATOR.save(store, creator)?;
        Ok(())
    }

    /// Get creator address if set
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an underlying storage issue.
    pub fn creator<Store: Storage>(store: &Store) -> StoreResult<Store, Option<String>> {
        CREATOR.may_load(store).map_err(Error::from)
    }

    /// Set the total rewards collected
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an underlying storage issue.
    pub fn set_total_rewards_collected<Store: MutStorage>(
        store: &mut Store,
        amount: u128,
    ) -> StoreResult<Store, ()> {
        REWARDS_COLLECTED.save(store, amount)?;
        Ok(())
    }

    /// Get total rewards collected
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an underlying storage issue.
    pub fn total_rewards_collected<Store: Storage>(store: &Store) -> StoreResult<Store, u128> {
        let collected = REWARDS_COLLECTED.may_load(store)?.unwrap_or_default();
        Ok(collected)
    }

    /// Set the number of rewards records collected
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an underlying storage issue.
    pub fn set_rewards_records_collected<Store: MutStorage>(
        store: &mut Store,
        amount: u64,
    ) -> StoreResult<Store, ()> {
        REWARDS_RECORDS_COLLECTED.save(store, amount)?;
        Ok(())
    }

    /// Get the number of rewards records collected
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an underlying storage issue.
    pub fn reward_records_collected<Store: Storage>(store: &Store) -> StoreResult<Store, u64> {
        let collected = REWARDS_RECORDS_COLLECTED
            .may_load(store)?
            .unwrap_or_default();
        Ok(collected)
    }
}
