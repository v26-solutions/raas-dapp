use cosmwasm_std::{entry_point, Binary, Env, MessageInfo, Reply};

use referrals_archway_drivers::{
    hub::{self, Error, ExecuteMsg, QueryMsg},
    Deps, DepsMut, Response,
};

#[entry_point]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    hub::execute(&mut deps, &env, info, msg)
}

#[entry_point]
pub fn reply(mut deps: DepsMut, env: Env, reply: Reply) -> Result<Response, Error> {
    hub::reply(&mut deps, &env, reply)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, Error> {
    hub::query(&deps, &env, &msg)
}
