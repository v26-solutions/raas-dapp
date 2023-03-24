use cosmwasm_std::{entry_point, Binary, Env, MessageInfo, Reply};

use referrals_archway_drivers::rewards_pot as driver;
use referrals_archway_drivers::{Deps, DepsMut};

use driver::{Error, ExecuteMsg, InstantiateMsg, QueryMsg, Response};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    driver::init(deps, env, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    driver::execute(deps, env, info, msg)
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, Error> {
    driver::reply(deps, env, reply)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, Error> {
    driver::query(deps, env, msg)
}
