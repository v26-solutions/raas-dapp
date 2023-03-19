#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use archway_bindings::{ArchwayMsg, ArchwayQuery};
use cosmwasm_std::{Deps as CwDeps, DepsMut as CwDepsMut, QuerierWrapper, Response as CwResponse};

use kv_storage::{Error as KvStoreError, KvStore};
use kv_storage_bincode::{Bincode, Error as BincodeError};
use kv_storage_cosmwasm::{Error as CwRepoError, Mutable, Readonly};

pub type MutStore<'a> = KvStore<Bincode, Mutable<'a>>;
pub type Store<'a> = KvStore<Bincode, Readonly<'a>>;
pub type StoreError = KvStoreError<BincodeError, CwRepoError>;
pub type Querier<'a> = QuerierWrapper<'a, ArchwayQuery>;
pub type Deps<'a> = CwDeps<'a, ArchwayQuery>;
pub type DepsMut<'a> = CwDepsMut<'a, ArchwayQuery>;
pub type Response = CwResponse<ArchwayMsg>;

pub mod hub;
pub mod rewards_pot;
