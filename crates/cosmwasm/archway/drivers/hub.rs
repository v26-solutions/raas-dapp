use archway_bindings::ArchwayMsg;
use cosmwasm_std::{
    Binary, Env, MessageInfo, Reply, StdError as CwStdError, Storage as CwStorage, SubMsg, WasmMsg,
};

use referrals_core::{Command as CoreCmd, Error as CoreError, Msg as CoreMsg, Reply as CoreReply};
use referrals_cw::rewards_pot::{ExecuteMsg as PotExecMsg, InstantiateMsg as PotInitMsg};
use referrals_cw::ReferralCodeResponse;
use referrals_parse_cw::Error as ParseError;

pub use referrals_cw::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::{Deps, DepsMut, Response, StoreError};

pub mod core_deps_impl;
pub mod state;

use core_deps_impl::{CoreDeps, Error as CoreDepsError};

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
}

fn core_exec(deps: &mut DepsMut, env: &Env, msg: CoreMsg) -> Result<CoreReply, Error> {
    let mut core_deps = CoreDeps::new(deps.storage, env, deps.querier);
    referrals_core::exec(&mut core_deps, msg).map_err(Error::from)
}

const CREATE_REWARDS_POT: u64 = 0;

fn add_cmd(
    storage: &mut dyn CwStorage,
    cmd: CoreCmd,
    response: Response,
) -> Result<Response, Error> {
    let response = match cmd {
        CoreCmd::CreateRewardsPot(dapp) => {
            let code_id = state::reward_pot_code_id(storage)?;
            let count = state::increment_reward_pot_count(storage)?;
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
            state::set_dapp_fee(storage, &dapp, amount)?;
            response
        }
    };

    Ok(response)
}

/// Translate a `referrals_core::Reply` into an archway cosmwasm `Response`.
///
/// # Errors
///
/// This function will return an error if:
/// - There is an issue with storage or cosmwasm serialisation
pub fn translate_core_reply(
    storage: &mut dyn CwStorage,
    reply: CoreReply,
) -> Result<Response, Error> {
    let response = Response::default();

    match reply {
        CoreReply::Empty => Ok(response),

        CoreReply::ReferralCode(code) => {
            let data = cosmwasm_std::to_binary(&ReferralCodeResponse {
                code: code.to_u64(),
            })?;

            Ok(response.set_data(data))
        }

        CoreReply::Cmd(cmd) => add_cmd(storage, cmd, response),

        CoreReply::MultiCmd(cmds) => cmds
            .into_iter()
            .try_fold(response, |response, cmd| add_cmd(storage, cmd, response)),
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
    state::set_reward_pot_code_id(deps.storage, msg.rewards_pot_code_id)?;
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
) -> Result<Response, Error> {
    referrals_parse_cw::parse_exec(deps.api, info, msg)
        .map_err(Error::from)
        .and_then(|msg| core_exec(deps, env, msg))
        .and_then(|reply| translate_core_reply(deps.storage, reply))
}

/// Handle the reply from any issued sub-messages.
///
/// # Errors
///
/// This function will return an error if the reply is invalid or there is a problem with `cosmwasm_std` storage or serialization
pub fn reply(deps: &mut DepsMut, env: &Env, reply: Reply) -> Result<Response, Error> {
    referrals_parse_cw::parse_init_pot_reply(reply)
        .map_err(Error::from)
        .and_then(|msg| core_exec(deps, env, msg))
        .and_then(|reply| translate_core_reply(deps.storage, reply))
}

/// Handle a `referrals_cw::QueryMsg`.
///
/// # Errors
///
/// This function should only return an error if there is a problem with `cosmwasm_std` storage or serialization
pub fn query(_deps: &Deps, _env: &Env, _msg: &QueryMsg) -> Result<Binary, Error> {
    Ok(Binary::default())
}
