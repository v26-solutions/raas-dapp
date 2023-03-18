#![deny(clippy::all)]
#![warn(clippy::pedantic)]

pub mod core_deps_impl;

use std::num::NonZeroU128;

use archway_bindings::ArchwayMsg;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError as CwStdError,
    Storage as CwStorage, SubMsg, WasmMsg,
};

use kv_storage::{item, map, Error as KvStoreError, Item, KvStore, Map};
use kv_storage_bincode::{Bincode, Error as BincodeError};
use kv_storage_cosmwasm::{Error as CwRepoError, Mutable, Readonly};

use referrals_core::{
    Command as CoreCmd, Error as CoreError, Id, Msg as CoreMsg, Reply as CoreReply,
};
use referrals_cw::rewards_pot::{ExecuteMsg as PotExecMsg, InstantiateMsg as PotInitMsg};
use referrals_cw::{ExecuteMsg, InstantiateMsg, QueryMsg, ReferralCodeResponse};
use referrals_parse_cw::Error as ParseError;

use core_deps_impl::{CoreDeps, Error as CoreDepsError};

pub(crate) type MutStore<'a> = KvStore<Bincode, Mutable<'a>>;
pub(crate) type Store<'a> = KvStore<Bincode, Readonly<'a>>;
pub(crate) type StoreError = KvStoreError<BincodeError, CwRepoError>;
// workaround for lack of flat-fees on constantine-1 testnet
// FIX: Next upgrade
pub(crate) type DappFeesMap<'a> = Map<1024, &'a str, NonZeroU128>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Storage(#[from] StoreError),
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Core(#[from] CoreError<CoreDepsError>),
    #[error(transparent)]
    CosmWasm(#[from] CwStdError),
    #[error("rewards pot code ID not set")]
    CodeIdNotSet,
}

static REWARD_POT_CODE_ID: Item<u64> = item!("reward_pot_code_id");
static REWARD_POT_COUNT: Item<u64> = item!("reward_pot_count");
static DAPP_FEES: DappFeesMap = map!("dapp_fees");

fn set_reward_pot_code_id(storage: &mut dyn CwStorage, code_id: u64) -> Result<(), Error> {
    REWARD_POT_CODE_ID.save(&mut MutStore::from_repo(storage), &code_id)?;
    Ok(())
}

fn reward_pot_code_id(storage: &dyn CwStorage) -> Result<u64, Error> {
    REWARD_POT_CODE_ID
        .may_load(&Store::from_repo(storage))?
        .ok_or(Error::CodeIdNotSet)
}

// workaround for lack of flat-fees on constantine-1 testnet
// FIX: Next upgrade
fn set_dapp_fee(storage: &mut dyn CwStorage, dapp: &Id, fee: NonZeroU128) -> Result<(), Error> {
    DAPP_FEES.save(&mut MutStore::from_repo(storage), &dapp.as_ref(), &fee)?;
    Ok(())
}

fn increment_reward_pot_count(storage: &mut dyn CwStorage) -> Result<u64, Error> {
    let mut storage = MutStore::from_repo(storage);
    let count = REWARD_POT_COUNT.may_load(&storage)?.unwrap_or_default();
    REWARD_POT_COUNT.save(&mut storage, &(count + 1))?;
    Ok(count)
}

fn core_exec(deps: &mut DepsMut, env: &Env, msg: CoreMsg) -> Result<CoreReply, Error> {
    let mut core_deps = CoreDeps::new(deps.storage, env, &*deps.querier, DAPP_FEES);
    referrals_core::exec(&mut core_deps, msg).map_err(Error::from)
}

const CREATE_REWARDS_POT: u64 = 0;

fn add_cmd(
    deps: &mut DepsMut,
    cmd: CoreCmd,
    response: Response<ArchwayMsg>,
) -> Result<Response<ArchwayMsg>, Error> {
    let response = match cmd {
        CoreCmd::CreateRewardsPot(dapp) => {
            let code_id = reward_pot_code_id(deps.storage)?;
            let count = increment_reward_pot_count(deps.storage)?;
            let msg = cosmwasm_std::to_binary(&PotInitMsg {
                dapp: dapp.into_string(),
            })?;

            response.add_submessage(SubMsg::reply_on_success(
                WasmMsg::Instantiate {
                    admin: None,
                    code_id,
                    msg,
                    funds: vec![],
                    label: format!("referrals-reward-pot-{count}"),
                },
                CREATE_REWARDS_POT,
            ))
        }

        CoreCmd::SetRewardsRecipient(recipient) => {
            response.add_message(ArchwayMsg::update_rewards_address(recipient.into_string()))
        }

        CoreCmd::SetRewardsAdmin(admin) => {
            response.add_message(ArchwayMsg::update_rewards_ownership(admin.into_string()))
        }

        CoreCmd::RedistributeRewards {
            amount,
            pot,
            receiver,
        } => {
            let msg = cosmwasm_std::to_binary(&PotExecMsg::DistributeRewards {
                recipient: receiver.into_string(),
                amount: amount.get().into(),
            })?;

            response.add_message(WasmMsg::Execute {
                contract_addr: pot.into_string(),
                msg,
                funds: vec![],
            })
        }

        CoreCmd::WithdrawPending(pot) => {
            let msg = cosmwasm_std::to_binary(&PotExecMsg::WithdrawRewards {})?;

            response.add_message(WasmMsg::Execute {
                contract_addr: pot.into_string(),
                msg,
                funds: vec![],
            })
        }

        CoreCmd::SetDappFee { dapp, amount } => {
            // workaround for lack of flat-fees on constantine-1 testnet
            // FIX: Next upgrade
            set_dapp_fee(deps.storage, &dapp, amount)?;
            response
        }
    };

    Ok(response)
}

fn handle_core_reply(deps: &mut DepsMut, reply: CoreReply) -> Result<Response<ArchwayMsg>, Error> {
    let response = Response::default();

    match reply {
        CoreReply::Empty => Ok(response),

        CoreReply::ReferralCode(code) => {
            let data = cosmwasm_std::to_binary(&ReferralCodeResponse {
                code: code.to_u64(),
            })?;

            Ok(response.set_data(data))
        }

        CoreReply::Cmd(cmd) => add_cmd(deps, cmd, response),

        CoreReply::MultiCmd(cmds) => cmds
            .into_iter()
            .try_fold(response, |response, cmd| add_cmd(deps, cmd, response)),
    }
}

/// Handle the `referrals_cw::InstantiateMsg`.
///
/// # Errors
///
/// This function will return an error if:
/// - There is an issue with storage
pub fn init(
    deps: &mut DepsMut,
    _env: &Env,
    _info: &MessageInfo,
    msg: &InstantiateMsg,
) -> Result<Response, Error> {
    set_reward_pot_code_id(deps.storage, msg.rewards_pot_code_id)?;
    Ok(Response::default())
}

/// Handle a `referrals_cw::ExecuteMsg`
///
/// # Errors
///
/// This function will return an error if:
/// - There is an issue parsing the input
/// - There is an issue in `referrals_core`
pub fn execute(
    deps: &mut DepsMut,
    env: &Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<ArchwayMsg>, Error> {
    referrals_parse_cw::parse_exec(deps.api, info, msg)
        .map_err(Error::from)
        .and_then(|msg| core_exec(deps, env, msg))
        .and_then(|reply| handle_core_reply(deps, reply))
}

/// Handle the reply from any issued sub-messages.
///
/// # Errors
///
/// This function will return an error if the original request cannot be completed
pub fn reply(deps: &mut DepsMut, env: &Env, reply: Reply) -> Result<Response<ArchwayMsg>, Error> {
    referrals_parse_cw::parse_init_pot_reply(reply)
        .map_err(Error::from)
        .and_then(|msg| core_exec(deps, env, msg))
        .and_then(|reply| handle_core_reply(deps, reply))
}

/// Handle a `referrals_cw::QueryMsg`.
///
/// # Errors
///
/// This function should only return an error if there is problem with `cosmwasm_std` storage or serialization
pub fn query(_deps: &Deps, _env: &Env, _msg: &QueryMsg) -> Result<Binary, Error> {
    Ok(Binary::default())
}
