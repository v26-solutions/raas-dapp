#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use cosmwasm_std::Storage as CwStorage;

use kv_storage::{item, Item};

use crate::{MutStore, Store, StoreError};

use super::Error;

static DAPP: Item<String> = item!("dapp");
static ADMIN: Item<String> = item!("admin");
static REWARDS_DENOM: Item<String> = item!("rewards_denom");
static REWARDS_COLLECTED: Item<u128> = item!("rewards_collected");
static REWARDS_RECORDS_COLLECTED: Item<u64> = item!("rewards_records_collected");

/// Set owner dApp address
///
/// # Errors
///
/// This function will return an error if there is an underlying storage issue.
pub fn set_dapp(storage: &mut dyn CwStorage, dapp: &String) -> Result<(), StoreError> {
    DAPP.save(&mut MutStore::from_repo(storage), dapp)?;
    Ok(())
}

/// Get owner dApp address
///
/// # Errors
///
/// This function will return an error if there is an underlying storage issue.
pub fn dapp(storage: &dyn CwStorage) -> Result<String, StoreError> {
    Ok(DAPP
        .may_load(&Store::from_repo(storage))?
        .expect("dapp set during initialisation"))
}

/// Set admin address
///
/// # Errors
///
/// This function will return an error if there is an underlying storage issue.
pub fn set_admin(storage: &mut dyn CwStorage, admin: &String) -> Result<(), StoreError> {
    ADMIN.save(&mut MutStore::from_repo(storage), admin)?;
    Ok(())
}

/// Get admin address
///
/// # Errors
///
/// This function will return an error if there is an underlying storage issue.
pub fn admin(storage: &dyn CwStorage) -> Result<String, StoreError> {
    Ok(DAPP
        .may_load(&Store::from_repo(storage))?
        .expect("admin set during initialisation"))
}

/// Checks if the given account is the admin
///
/// # Panics
///
/// Panics if the admin was not set during initialisation.
///
/// # Errors
///
/// This function will return an error if there is an underlying storage issue.
pub fn is_admin(storage: &dyn CwStorage, account: &str) -> Result<bool, StoreError> {
    Ok(DAPP
        .may_load(&Store::from_repo(storage))?
        .expect("admin set during initialisation")
        == account)
}

/// Checks if the reward coin denom is set
///
/// # Errors
///
/// This function will return an error if there is an underlying storage issue.
pub fn rewards_denom_is_set(storage: &dyn CwStorage) -> Result<bool, StoreError> {
    REWARDS_DENOM
        .is_empty(&Store::from_repo(storage))
        .map(|is_empty| !is_empty)
}

/// Set the reward coin denom
///
/// # Errors
///
/// This function will return an error if there is an underlying storage issue.
pub fn set_rewards_denom(storage: &mut dyn CwStorage, denom: &String) -> Result<(), StoreError> {
    REWARDS_DENOM.save(&mut MutStore::from_repo(storage), denom)?;
    Ok(())
}

/// Get the reward coin denom
///
/// # Errors
///
/// This function will return an error if there is an underlying storage issue.
pub fn rewards_denom(storage: &dyn CwStorage) -> Result<String, StoreError> {
    Ok(REWARDS_DENOM
        .may_load(&Store::from_repo(storage))?
        .expect("rewards denom set after first successful rewards withdrawal"))
}

/// Add an amount onto the total rewards collected
///
/// # Errors
///
/// This function will return an error if there is an underlying storage issue.
pub fn add_reward_collection(storage: &mut dyn CwStorage, amount: u128) -> Result<(), Error> {
    let mut storage = MutStore::from_repo(storage);

    let collected = REWARDS_COLLECTED
        .may_load(&storage)?
        .unwrap_or_default()
        .checked_add(amount)
        .ok_or(Error::RewardsOverflow)?;

    REWARDS_COLLECTED.save(&mut storage, &collected)?;

    Ok(())
}

/// Get total rewards collected
///
/// # Errors
///
/// This function will return an error if there is an underlying storage issue.
pub fn total_rewards_collected(storage: &dyn CwStorage) -> Result<u128, StoreError> {
    let collected = REWARDS_COLLECTED
        .may_load(&Store::from_repo(storage))?
        .unwrap_or_default();

    Ok(collected)
}

/// Set the number of rewards records collected
///
/// # Errors
///
/// This function will return an error if there is an underlying storage issue.
pub fn set_rewards_records_collected(
    storage: &mut dyn CwStorage,
    amount: u64,
) -> Result<(), StoreError> {
    REWARDS_RECORDS_COLLECTED.save(&mut MutStore::from_repo(storage), &amount)?;
    Ok(())
}

/// Get the number of rewards records collected
///
/// # Errors
///
/// This function will return an error if there is an underlying storage issue.
pub fn reward_records_collected(storage: &dyn CwStorage) -> Result<u64, StoreError> {
    let collected = REWARDS_RECORDS_COLLECTED
        .may_load(&Store::from_repo(storage))?
        .unwrap_or_default();

    Ok(collected)
}
