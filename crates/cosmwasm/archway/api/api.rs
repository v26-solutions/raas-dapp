#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use archway_bindings::{ArchwayMsg, ArchwayQuery};
use cosmwasm_std::{Env, QuerierWrapper, Response as CwResponse, StdError, Storage as CwStorage};

use kv_storage::{Error as KvStoreError, KvStore, MutStorage};
use kv_storage_bincode::{Bincode, Error as BincodeError};
use kv_storage_cosmwasm::{CosmwasmRepo, Error as CosmwasmRepoError};

use referrals_core::FallibleApi;

use referrals_storage::Error as CoreStorageError;
use referrals_storage::Storage as CoreStorage;

pub type Querier<'a> = QuerierWrapper<'a, ArchwayQuery>;
pub type Response = CwResponse<ArchwayMsg>;

pub type CwStore<'a> = KvStore<Bincode, CosmwasmRepo<&'a dyn CwStorage>>;
pub type CwMutStore<'a> = KvStore<Bincode, CosmwasmRepo<&'a mut dyn CwStorage>>;
pub type CwStoreError = KvStoreError<BincodeError, CosmwasmRepoError>;

pub mod cache;
pub mod hub;
pub mod rewards_pot;

#[derive(Debug, thiserror::Error)]
pub enum Error<StoreError, ModeError> {
    #[error(transparent)]
    CoreStorage(#[from] CoreStorageError<StoreError>),
    #[error(transparent)]
    CosmWasmStd(#[from] StdError),
    #[error(transparent)]
    Cache(#[from] cache::Error<StoreError>),
    #[error(transparent)]
    Mode(ModeError),
}

pub struct Api<'a, Mode, Store> {
    store: Store,
    env: &'a Env,
    querier: Querier<'a>,
    response: Response,
    _m: Mode,
}

impl<'a, Mode, Store> Api<'a, Mode, Store> {
    #[must_use]
    pub fn core_storage(&self) -> CoreStorage<&Store> {
        CoreStorage::new(&self.store)
    }

    pub fn core_storage_mut(&mut self) -> CoreStorage<&mut Store> {
        CoreStorage::new(&mut self.store)
    }
}

impl<'a, Mode, Store> Api<'a, Mode, Store>
where
    Mode: Default,
{
    pub fn new(store: Store, env: &'a Env, querier: Querier<'a>) -> Self {
        Self {
            store,
            env,
            querier,
            response: Response::default(),
            _m: Mode::default(),
        }
    }
}

impl<'a, Mode, Store> Api<'a, Mode, Store>
where
    Mode: FallibleApi,
    Store: MutStorage,
{
    /// Get the rewards denom, either from the cache or query
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with underlying storage.
    pub fn rewards_denom(&mut self) -> Result<String, Error<Store::Error, Mode::Error>> {
        let Some(cached) = cache::rewards_denom(&self.store)? else {
            let denom = self.querier.query_bonded_denom()?;
            cache::set_rewards_denom(&mut self.store, &denom)?;
            return Ok(denom)
        };

        Ok(cached)
    }
}
