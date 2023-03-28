use cosmwasm_std::{Binary, Env, MessageInfo, Reply, StdError};

use referrals_archway::ResponseExt;
use referrals_cw::{ExecuteMsg as HubExecuteMsg, WithReferralCode};
use referrals_parse_cw::Error as ParseError;

use referrals_archway_api::hub as api;
use referrals_core::hub as _core;

use _core::Error as CoreError;
use api::CwApiError;

pub use referrals_archway_api::Response;
pub use referrals_cw::{InstantiateMsg, QueryMsg};

pub type ExecuteMsg = WithReferralCode<HubExecuteMsg>;

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

/// Handle the `referrals_cw::InstantiateMsg`.
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
    api::from_deps_mut(&mut deps, &env).initialize(msg.rewards_pot_code_id)?;

    Response::default()
        .activate_dapp_referrals()
        .referral_hub(env.contract.address)
        .dapp_name("referrals_hub")
        .referrer_percent(100)
        .collector(info.sender)
        .done()
        .map_err(Error::from)
}

/// Handle a `referrals_cw::ExecuteMsg`
///
/// # Errors
///
/// This function will return an error if:
/// - There is an issue parsing the input
/// - There is an issue in `referrals_core`
#[allow(clippy::needless_pass_by_value)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    let core_msg = referrals_parse_cw::parse_hub_exec(deps.api, info, msg.msg)?;

    let mut api = api::from_deps_mut(&mut deps, &env);

    let reply = _core::exec(&mut api, core_msg)?;

    let response = _core::handle_reply(api, reply)?;

    let Some(code) = msg.referral_code else {
        return Ok(response);
    };

    response
        .record_referral()
        .referral_code(code)
        .referral_hub(env.contract.address)
        .done()
        .map_err(Error::from)
}

/// Handle the reply from any issued sub-messages.
///
/// # Errors
///
/// This function will return an error if the reply is invalid or there is a problem with `cosmwasm_std` storage or serialization
#[allow(clippy::needless_pass_by_value)]
pub fn reply(mut deps: DepsMut, env: Env, reply: Reply) -> Result<Response, Error> {
    let mut api = api::from_deps_mut(&mut deps, &env);

    let msg = referrals_parse_cw::parse_init_pot_reply(reply)?;

    let reply = _core::exec(&mut api, msg)?;

    _core::handle_reply(api, reply).map_err(Error::from)
}

/// Handle a `referrals_cw::QueryMsg`.
///
/// # Errors
///
/// This function should only return an error if there is a problem with `cosmwasm_std` storage or serialization
#[allow(clippy::needless_pass_by_value)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> Result<Binary, Error> {
    Ok(Binary::default())
}
