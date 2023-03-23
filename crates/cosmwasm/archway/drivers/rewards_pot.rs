use cosmwasm_std::{Binary, Env, MessageInfo, Reply, StdError};

use referrals_parse_cw::Error as ParseError;

use referrals_archway_api::rewards_pot as api;
use referrals_core::rewards_pot as _core;
use referrals_cw::rewards_pot::InstantiateResponse;

use _core::Error as CoreError;
use api::CwApiError;

pub use referrals_archway_api::Response;
pub use referrals_cw::rewards_pot::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::{Deps, DepsMut};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Api(#[from] CwApiError),
    #[error(transparent)]
    Core(#[from] CoreError<CwApiError>),
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    CosmWasm(#[from] StdError),
}

/// Handle the rewards-pot `InstantiateMsg`.
///
/// # Errors
///
/// This function will return an error if:
/// - There is an issue with storage
#[allow(clippy::needless_pass_by_value)]
pub fn init(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    api::from_deps_mut(&mut deps, &env).initialize(info.sender, &msg.dapp)?;

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
#[allow(clippy::needless_pass_by_value)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    let core_msg = referrals_parse_cw::parse_pot_exec(deps.api, info, msg)?;

    let mut api = api::from_deps_mut(&mut deps, &env);

    let reply = _core::exec(&mut api, core_msg)?;

    let response = _core::handle_reply(api, reply)?;

    Ok(response)
}

/// Handle the reply from any issued sub-messages.
///
/// # Errors
///
/// This function will return an error if the original request cannot be completed
#[allow(clippy::needless_pass_by_value)]
pub fn reply(mut deps: DepsMut, env: Env, reply: Reply) -> Result<Response, Error> {
    api::from_deps_mut(&mut deps, &env).handle_cw_reply(reply)?;

    Ok(Response::default())
}

/// Handle a rewards-pot `QueryMsg`.
///
/// # Errors
///
/// This function should only return an error if there is problem with `cosmwasm_std` storage or serialization
#[allow(clippy::needless_pass_by_value)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, Error> {
    let api = api::from_deps(deps, &env);

    let response = match msg {
        QueryMsg::TotalRewards {} => {
            let total_rewards = api.total_rewards()?;
            cosmwasm_std::to_binary(&total_rewards)?
        }

        QueryMsg::Dapp {} => {
            let dapp = api.dapp()?;
            cosmwasm_std::to_binary(&dapp)?
        }

        QueryMsg::Admin {} => {
            let admin = api.admin()?;
            cosmwasm_std::to_binary(&admin)?
        }
    };

    Ok(response)
}
