use std::num::NonZeroU128;

use cosmwasm_std::Storage as CwStorage;

use kv_storage::{item, map, Item, Map};

use referrals_core::Id;

use crate::{MutStore, Store, StoreError};

static REWARD_POT_CODE_ID: Item<u64> = item!("reward_pot_code_id");
static REWARD_POT_COUNT: Item<u64> = item!("reward_pot_count");
static DAPP_FEES: Map<1024, &str, NonZeroU128> = map!("dapp_fees");

/// Set the reward pot contract code id
///
/// # Errors
///
/// This function will return an error if there is an issue with the underlying storage.
pub fn set_reward_pot_code_id(storage: &mut dyn CwStorage, code_id: u64) -> Result<(), StoreError> {
    REWARD_POT_CODE_ID.save(&mut MutStore::from_repo(storage), &code_id)?;
    Ok(())
}

/// Get the reward pot contract code id
///
/// # Errors
///
/// This function will return an error if there is an issue with the underlying storage.
pub fn reward_pot_code_id(storage: &dyn CwStorage) -> Result<u64, StoreError> {
    Ok(REWARD_POT_CODE_ID
        .may_load(&Store::from_repo(storage))?
        .expect("code id set during initialisation"))
}

// workarounds for lack of flat-fees on constantine-1 testnet
// FIX: Next upgrade

/// Set the dapp's flat fee
///
/// # Errors
///
/// This function will return an error if there is an issue with the underlying storage.
pub fn set_dapp_fee(
    storage: &mut dyn CwStorage,
    dapp: &Id,
    fee: NonZeroU128,
) -> Result<(), StoreError> {
    DAPP_FEES.save(&mut MutStore::from_repo(storage), &dapp.as_ref(), &fee)?;
    Ok(())
}

/// Get the dapp's flat fee
///
/// # Errors
///
/// This function will return an error if there is an issue with the underlying storage.
pub fn dapp_fee(storage: &dyn CwStorage, dapp: &Id) -> Result<Option<NonZeroU128>, StoreError> {
    DAPP_FEES
        .may_load(&Store::from_repo(storage), &dapp.as_ref())
        .map_err(StoreError::from)
}

/// Increment the number of reward pots created, returning the new value.
///
/// # Errors
///
/// This function will return an error if there is an issue with the underlying storage.
pub fn increment_reward_pot_count(storage: &mut dyn CwStorage) -> Result<u64, StoreError> {
    let mut storage = MutStore::from_repo(storage);
    let count = REWARD_POT_COUNT.may_load(&storage)?.unwrap_or_default();
    REWARD_POT_COUNT.save(&mut storage, &(count + 1))?;
    Ok(count)
}
