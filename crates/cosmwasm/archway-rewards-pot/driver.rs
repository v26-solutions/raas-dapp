#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use archway_bindings::{
    types::rewards::{RewardsRecordsResponse, WithdrawRewardsResponse},
    ArchwayMsg, ArchwayQuery, PageRequest,
};
use cosmwasm_std::{
    coins, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, Reply, Response,
    StdError as CwStdError, Storage as CwStorage, SubMsg,
};

use kv_storage::{item, Error as KvStoreError, Item, KvStore};
use kv_storage_bincode::{Bincode, Error as BincodeError};
use kv_storage_cosmwasm::{Error as CwRepoError, Mutable, Readonly};

use referrals_cw::rewards_pot::{
    AdminResponse, DappResponse, ExecuteMsg, InstantiateMsg, InstantiateResponse, QueryMsg,
    TotalRewardsResponse,
};

type MutStore<'a> = KvStore<Bincode, Mutable<'a>>;
type Store<'a> = KvStore<Bincode, Readonly<'a>>;
type StoreError = KvStoreError<BincodeError, CwRepoError>;
type Querier<'a> = QuerierWrapper<'a, ArchwayQuery>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Storage(#[from] StoreError),
    #[error(transparent)]
    CosmWasm(#[from] CwStdError),
    #[error("dApp is not set")]
    DappNotSet,
    #[error("admin is not set")]
    AdminNotSet,
    #[error("unauthorized")]
    Unauthorized,
    #[error("rewards denom is not set")]
    RewardsDenomNotSet,
    #[error("unexpected error reply")]
    UnexpectedErrorReply(String),
    #[error("expected reply data")]
    ExptectedReplyData,
    #[error("rewards overflow")]
    // oh to be so lucky
    RewardsOverflow,
}

static ADMIN: Item<String> = item!("admin");
static DAPP: Item<String> = item!("dapp");
static REWARDS_DENOM: Item<String> = item!("rewards_denom");
static REWARDS_RECORDS_COLLECTED: Item<u64> = item!("rewards_records_collected");
static REWARDS_COLLECTED: Item<u128> = item!("rewards_collected");

fn set_dapp(storage: &mut dyn CwStorage, dapp: &String) -> Result<(), Error> {
    DAPP.save(&mut MutStore::from_repo(storage), dapp)?;
    Ok(())
}

fn dapp(storage: &dyn CwStorage) -> Result<String, Error> {
    DAPP.may_load(&Store::from_repo(storage))?
        .ok_or(Error::DappNotSet)
}

fn set_admin(storage: &mut dyn CwStorage, admin: &String) -> Result<(), Error> {
    ADMIN.save(&mut MutStore::from_repo(storage), admin)?;
    Ok(())
}

fn admin(storage: &dyn CwStorage) -> Result<String, Error> {
    DAPP.may_load(&Store::from_repo(storage))?
        .ok_or(Error::AdminNotSet)
}

fn is_authorized(storage: &dyn CwStorage, account: &str) -> Result<bool, Error> {
    DAPP.may_load(&Store::from_repo(storage))?
        .ok_or(Error::AdminNotSet)
        .map(|admin| admin == account)
}

fn rewards_denom_is_set(storage: &dyn CwStorage) -> Result<bool, Error> {
    REWARDS_DENOM
        .is_empty(&Store::from_repo(storage))
        .map_err(Error::from)
        .map(|is_empty| !is_empty)
}

fn set_rewards_denom(storage: &mut dyn CwStorage, denom: &String) -> Result<(), Error> {
    REWARDS_DENOM.save(&mut MutStore::from_repo(storage), denom)?;
    Ok(())
}

fn rewards_denom(storage: &dyn CwStorage) -> Result<String, Error> {
    REWARDS_DENOM
        .may_load(&Store::from_repo(storage))?
        .ok_or(Error::RewardsDenomNotSet)
}

fn set_rewards_records_collected(storage: &mut dyn CwStorage, amount: u64) -> Result<(), Error> {
    REWARDS_RECORDS_COLLECTED.save(&mut MutStore::from_repo(storage), &amount)?;
    Ok(())
}

fn reward_records_collected(storage: &dyn CwStorage) -> Result<u64, Error> {
    let collected = REWARDS_RECORDS_COLLECTED
        .may_load(&Store::from_repo(storage))?
        .unwrap_or_default();

    Ok(collected)
}

fn record_collection(storage: &mut dyn CwStorage, amount: u128) -> Result<(), Error> {
    let mut storage = MutStore::from_repo(storage);

    let collected = REWARDS_COLLECTED
        .may_load(&storage)?
        .unwrap_or_default()
        .checked_add(amount)
        .ok_or(Error::RewardsOverflow)?;

    REWARDS_COLLECTED.save(&mut storage, &collected)?;

    Ok(())
}

fn rewards_collected(storage: &dyn CwStorage) -> Result<u128, Error> {
    let collected = REWARDS_COLLECTED
        .may_load(&Store::from_repo(storage))?
        .unwrap_or_default();

    Ok(collected)
}

fn total_rewards_records(env: &Env, querier: &QuerierWrapper<ArchwayQuery>) -> Result<u64, Error> {
    let rewards_records_response: RewardsRecordsResponse = querier.query(
        &ArchwayQuery::rewards_records_with_pagination(
            &env.contract.address,
            PageRequest::new().limit(1),
        )
        .into(),
    )?;

    let total_records = rewards_records_response
        .pagination
        .and_then(|page| page.total)
        .unwrap_or_default();

    Ok(total_records)
}

struct OutstandingRecords {
    total_records: u64,
    pending_records: u64,
}

fn outstanding_records(
    storage: &dyn CwStorage,
    env: &Env,
    querier: &Querier,
) -> Result<OutstandingRecords, Error> {
    let records_collected = reward_records_collected(storage)?;
    let total_records = total_rewards_records(env, querier)?;
    let pending_records = total_records.saturating_sub(records_collected);

    Ok(OutstandingRecords {
        total_records,
        pending_records,
    })
}

/// Handle the rewards-pot `InstantiateMsg`.
///
/// # Errors
///
/// This function will return an error if:
/// - There is an issue with storage
pub fn init(
    deps: &mut DepsMut,
    _env: &Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    set_dapp(deps.storage, &msg.dapp)?;
    set_admin(deps.storage, &info.sender.into_string())?;

    let data = cosmwasm_std::to_binary(&InstantiateResponse { dapp: msg.dapp })?;

    Ok(Response::default().set_data(data))
}

/// Handle a rewards-pot `ExecuteMsg`
///
/// # Errors
///
/// This function will return an error if:
/// - The sender is not the admin (initiator) of the contract
/// - The rewards distribution recipient is not a valid address
pub fn execute(
    deps: &mut DepsMut<ArchwayQuery>,
    env: &Env,
    info: &MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<ArchwayMsg>, Error> {
    if !is_authorized(deps.storage, info.sender.as_str())? {
        return Err(Error::Unauthorized)?;
    }

    let response = Response::default();

    let response = match msg {
        ExecuteMsg::WithdrawRewards {} => {
            let outstanding_records = outstanding_records(deps.storage, env, &deps.querier)?;

            if outstanding_records.pending_records > 0 {
                set_rewards_records_collected(deps.storage, outstanding_records.total_records)?;
                response.add_submessage(SubMsg::reply_on_success(
                    ArchwayMsg::withdraw_rewards_by_limit(outstanding_records.pending_records),
                    0,
                ))
            } else {
                response
            }
        }

        ExecuteMsg::DistributeRewards { recipient, amount } => {
            deps.api.addr_validate(&recipient)?;

            let rewards_denom = rewards_denom(deps.storage)?;

            response.add_message(BankMsg::Send {
                to_address: recipient,
                amount: coins(amount.u128(), rewards_denom),
            })
        }
    };

    Ok(response)
}

/// Handle the reply from any issued sub-messages.
///
/// # Errors
///
/// This function will return an error if the original request cannot be completed
pub fn reply(deps: &mut DepsMut, _env: &Env, reply: Reply) -> Result<Response<ArchwayMsg>, Error> {
    let response = reply
        .result
        .into_result()
        .map_err(Error::UnexpectedErrorReply)?;

    // the only sub-message issued is rewards withdrawal
    let withdraw_response: WithdrawRewardsResponse = response
        .data
        .ok_or(Error::ExptectedReplyData)
        .and_then(|b| cosmwasm_std::from_binary(&b).map_err(Error::from))?;

    let Some(rewards) = withdraw_response.total_rewards.first() else {
        return Ok(Response::default());
    };

    if rewards_denom_is_set(deps.storage)? {
        set_rewards_denom(deps.storage, &rewards.denom)?;
    }

    record_collection(deps.storage, rewards.amount.u128())?;

    Ok(Response::default())
}

/// Handle a rewards-pot `QueryMsg`.
///
/// # Errors
///
/// This function should only return an error if there is problem with `cosmwasm_std` storage or serialization
pub fn query(deps: &Deps<ArchwayQuery>, env: &Env, msg: &QueryMsg) -> Result<Binary, Error> {
    let response = match msg {
        QueryMsg::TotalRewards {} => {
            let rewards_collected = rewards_collected(deps.storage)?;

            let outstanding_records = outstanding_records(deps.storage, env, &deps.querier)?;

            let rewards_records_response: RewardsRecordsResponse = deps.querier.query(
                &ArchwayQuery::rewards_records_with_pagination(
                    &env.contract.address,
                    PageRequest::new().limit(outstanding_records.pending_records),
                )
                .into(),
            )?;

            let total = rewards_records_response
                .records
                .into_iter()
                .flat_map(|record| record.rewards)
                .try_fold(rewards_collected, |total, reward| {
                    total
                        .checked_add(reward.amount.u128())
                        .ok_or(Error::RewardsOverflow)
                })?;

            cosmwasm_std::to_binary(&TotalRewardsResponse {
                total: total.into(),
            })?
        }

        QueryMsg::Dapp {} => {
            let dapp = dapp(deps.storage)?;
            cosmwasm_std::to_binary(&DappResponse { dapp })?
        }

        QueryMsg::Admin {} => {
            let admin = admin(deps.storage)?;
            cosmwasm_std::to_binary(&AdminResponse { admin })?
        }
    };

    Ok(response)
}
