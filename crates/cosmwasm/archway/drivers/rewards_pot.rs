use archway_bindings::{
    types::rewards::{RewardsRecordsResponse, WithdrawRewardsResponse},
    ArchwayMsg, ArchwayQuery, PageRequest,
};
use cosmwasm_std::{
    coins, BankMsg, Binary, Env, MessageInfo, Reply, StdError as CwStdError, Storage as CwStorage,
    SubMsg,
};

use referrals_cw::rewards_pot::{
    AdminResponse, DappResponse, InstantiateResponse, TotalRewardsResponse,
};

pub use referrals_cw::rewards_pot::{ExecuteMsg, InstantiateMsg, QueryMsg};

pub mod state;

use crate::{Deps, DepsMut, Querier, Response, StoreError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Storage(#[from] StoreError),
    #[error(transparent)]
    CosmWasm(#[from] CwStdError),
    #[error("unauthorized")]
    Unauthorized,
    #[error("no rewards withdrawn")]
    NoRewardsWithdrawn,
    #[error("unexpected error reply - {0}")]
    UnexpectedErrorReply(String),
    #[error("expected reply data")]
    ExptectedReplyData,
    #[error("rewards overflow")]
    RewardsOverflow,
}

/// Query the rewards pots total rewards records count
///
/// # Errors
///
/// This function will return an error if there is an issue with the underlying querier.
pub fn total_rewards_records(env: &Env, querier: Querier) -> Result<u64, Error> {
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

pub struct OutstandingRecords {
    total_records: u64,
    pending_records: u64,
}

/// Calculate how many outstanding rewards records the rewards pot has.
///
/// # Errors
///
/// This function will return an error if there is an issue with the underlying querier or storage.
pub fn outstanding_records(
    storage: &dyn CwStorage,
    env: &Env,
    querier: Querier,
) -> Result<OutstandingRecords, Error> {
    let records_collected = state::reward_records_collected(storage)?;
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
    state::set_dapp(deps.storage, &msg.dapp)?;
    state::set_admin(deps.storage, &info.sender.into_string())?;

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
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    if !state::is_admin(deps.storage, info.sender.as_str())? {
        return Err(Error::Unauthorized)?;
    }

    let response = Response::default();

    let response = match msg {
        ExecuteMsg::WithdrawRewards {} => {
            let outstanding_records = outstanding_records(deps.storage, env, deps.querier)?;

            if outstanding_records.pending_records > 0 {
                state::set_rewards_records_collected(
                    deps.storage,
                    outstanding_records.total_records,
                )?;

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

            if !state::rewards_denom_is_set(deps.storage)? {
                return Err(Error::NoRewardsWithdrawn);
            }

            let rewards_denom = state::rewards_denom(deps.storage)?;

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
pub fn reply(deps: &mut DepsMut, _env: &Env, reply: Reply) -> Result<Response, Error> {
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

    if !state::rewards_denom_is_set(deps.storage)? {
        state::set_rewards_denom(deps.storage, &rewards.denom)?;
    }

    state::add_reward_collection(deps.storage, rewards.amount.u128())?;

    Ok(Response::default())
}

/// Handle a rewards-pot `QueryMsg`.
///
/// # Errors
///
/// This function should only return an error if there is problem with `cosmwasm_std` storage or serialization
pub fn query(deps: &Deps, env: &Env, msg: &QueryMsg) -> Result<Binary, Error> {
    let response = match msg {
        QueryMsg::TotalRewards {} => {
            let rewards_collected = state::total_rewards_collected(deps.storage)?;

            let outstanding_records = outstanding_records(deps.storage, env, deps.querier)?;

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
            let dapp = state::dapp(deps.storage)?;
            cosmwasm_std::to_binary(&DappResponse { dapp })?
        }

        QueryMsg::Admin {} => {
            let admin = state::admin(deps.storage)?;
            cosmwasm_std::to_binary(&AdminResponse { admin })?
        }
    };

    Ok(response)
}
