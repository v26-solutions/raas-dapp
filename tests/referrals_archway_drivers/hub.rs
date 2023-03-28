use archway_bindings::testing::MockDepsExt;
use archway_bindings::types::rewards::{ContractMetadataResponse, FlatFeeResponse};
use archway_bindings::ArchwayQuery;
use cosmwasm_std::{
    to_binary, Addr, ContractResult, QuerierResult, QueryResponse, Uint128, WasmQuery,
};
use referrals_archway_api::hub as api;
use referrals_archway_drivers::hub;
use referrals_archway_drivers::hub::InstantiateMsg;
use referrals_archway_drivers::rewards_pot::{
    ExecuteMsg as PotExecuteMsg, InstantiateMsg as PotInitMsg, QueryMsg as PotQueryMsg,
};
use referrals_core::hub::{self as hub_core, Kind, Msg, Registration};
use referrals_core::Id;
use referrals_cw::rewards_pot::{AdminResponse, DappResponse, TotalRewardsResponse};
use referrals_cw::{ExecuteMsg, ReferralCodeResponse, WithReferralCode};

use crate::{check, expect, pretty};

use super::DisplayResponse;

pub fn wasm_query_handler(query: &WasmQuery) -> QuerierResult {
    match query {
        WasmQuery::Smart { contract_addr, msg } => {
            assert_eq!(contract_addr, "rewards_pot_0");
            let msg: PotQueryMsg = cosmwasm_std::from_binary(msg).unwrap();

            let res = match msg {
                PotQueryMsg::TotalRewards {} => cosmwasm_std::to_binary(&TotalRewardsResponse {
                    total: Uint128::new(5000),
                }),
                PotQueryMsg::Dapp {} => cosmwasm_std::to_binary(&DappResponse {
                    dapp: "dapp".to_owned(),
                }),
                PotQueryMsg::Admin {} => cosmwasm_std::to_binary(&AdminResponse {
                    admin: "referrals_hub".to_owned(),
                }),
            }
            .unwrap();

            QuerierResult::Ok(ContractResult::Ok(res))
        }
        _ => panic!("unhandled query: {query:?}"),
    }
}

pub fn archway_query_handler(
    query: &ArchwayQuery,
    flat_fee: u128,
) -> ContractResult<QueryResponse> {
    let response = match query {
        ArchwayQuery::ContractMetadata { .. } => to_binary(&ContractMetadataResponse {
            owner_address: String::from("referrals_hub"),
            rewards_address: String::from("referrals_hub"),
        }),
        ArchwayQuery::FlatFee { .. } => to_binary(&FlatFeeResponse {
            flat_fee_amount: cosmwasm_std::Coin::new(flat_fee, "test"),
        }),
        _ => panic!("unhandled archway query: {query:?}"),
    };

    response.into()
}

macro_rules! info {
    ($sender:literal) => {
        cosmwasm_std::testing::mock_info($sender, &[])
    };
}

macro_rules! env {
    () => {{
        let mut env = cosmwasm_std::testing::mock_env();
        env.contract.address = Addr::unchecked("referrals_hub");
        env
    }};
}

macro_rules! do_ok {
    ($op:ident, $deps:ident, $env:expr, $from:expr, $msg:expr) => {{
        hub::$op($deps.as_mut(), $env, $from, $msg)
            .map(DisplayResponse::from)
            .unwrap()
    }};
}

macro_rules! init_ok {
    ($deps:ident, $from:literal, $msg:expr) => {
        do_ok!(init, $deps, env!(), info!($from), $msg)
    };
}

macro_rules! exec_ok {
    ($deps:ident, $from:literal, $msg:expr) => {
        do_ok!(
            execute,
            $deps,
            env!(),
            info!($from),
            WithReferralCode::from($msg)
        )
    };
}

#[test]
fn plumbing_works() {
    let mut deps =
        archway_bindings::testing::mock_dependencies(move |q| archway_query_handler(q, 0));

    deps.querier.update_wasm(wasm_query_handler);

    let res: DisplayResponse<(), ExecuteMsg> = init_ok!(
        deps,
        "hub_owner",
        InstantiateMsg {
            rewards_pot_code_id: 1,
        }
    );

    check(
        pretty(&res),
        expect![[r#"
            (
              data: None,
              messages: [
                (
                  id: 0,
                  msg: Std(custom(update_contract_metadata(
                    contract_address: None,
                    owner_address: Some("referrals_hub"),
                    rewards_address: None,
                  ))),
                  reply_on: never,
                ),
                (
                  id: 0,
                  msg: Wasm(Execute(
                    contract_addr: "referrals_hub",
                    msg: activate_dapp(
                      name: "referrals_hub",
                      percent: 100,
                      collector: "hub_owner",
                    ),
                  )),
                  reply_on: never,
                ),
              ],
              attributes: [],
              events: [],
            )"#]],
    );

    let res: DisplayResponse<ReferralCodeResponse> =
        exec_ok!(deps, "referrer", ExecuteMsg::RegisterReferrer {});

    check(
        pretty(&res),
        expect![[r#"
            (
              data: Some((
                code: 1,
              )),
              messages: [],
              attributes: [],
              events: [],
            )"#]],
    );

    let res: DisplayResponse<(), PotInitMsg> = exec_ok!(
        deps,
        "dapp",
        ExecuteMsg::ActivateDapp {
            name: "dapp".to_owned(),
            percent: 75,
            collector: "collector".to_owned(),
        }
    );

    check(
        pretty(&res),
        expect![[r#"
            (
              data: None,
              messages: [
                (
                  id: 0,
                  msg: Wasm(Instantiate(
                    code_id: 1,
                    msg: (
                      dapp: "dapp",
                    ),
                    label: "referrals-reward-pot-0",
                  )),
                  reply_on: success,
                ),
              ],
              attributes: [],
              events: [],
            )"#]],
    );

    // Skip Instanitate Reply parsing and set rewards pot address directly
    {
        let env = env!();
        let mut deps = deps.as_mut();
        let mut api = api::from_deps_mut(&mut deps, &env);
        hub_core::exec(
            &mut api,
            Msg {
                sender: Id::from("referrals_hub"),
                kind: Kind::Register(Registration::RewardsPot {
                    dapp: Id::from("dapp"),
                    rewards_pot: Id::from("rewards_pot_0"),
                }),
            },
        )
        .unwrap();
    }

    let res: DisplayResponse = exec_ok!(
        deps,
        "dapp",
        ExecuteMsg::SetDappFee {
            dapp: "dapp".to_owned(),
            fee: Uint128::new(1000),
        }
    );

    check(
        pretty(&res),
        expect![[r#"
            (
              data: None,
              messages: [
                (
                  id: 0,
                  msg: Std(custom(set_flat_fee(
                    contract_address: Some("dapp"),
                    flat_fee_amount: (
                      denom: "",
                      amount: "1000",
                    ),
                  ))),
                  reply_on: never,
                ),
              ],
              attributes: [],
              events: [],
            )"#]],
    );

    let mut deps = deps.with_archway_query_handler(move |q| archway_query_handler(q, 1000));

    let res: DisplayResponse = exec_ok!(deps, "dapp", ExecuteMsg::RecordReferral { code: 1 });

    check(
        pretty(&res),
        expect![[r#"
            (
              data: None,
              messages: [],
              attributes: [],
              events: [],
            )"#]],
    );

    let res: DisplayResponse = exec_ok!(
        deps,
        "referrer",
        ExecuteMsg::TransferOwnership {
            code: 1,
            owner: "referrer_new".to_owned(),
        }
    );

    check(
        pretty(&res),
        expect![[r#"
            (
              data: None,
              messages: [],
              attributes: [],
              events: [],
            )"#]],
    );

    let res: DisplayResponse<(), PotExecuteMsg> = exec_ok!(
        deps,
        "referrer_new",
        ExecuteMsg::CollectReferrer {
            code: 1,
            dapp: "dapp".to_owned(),
        }
    );

    check(
        pretty(&res),
        expect![[r#"
            (
              data: None,
              messages: [
                (
                  id: 0,
                  msg: Wasm(Execute(
                    contract_addr: "rewards_pot_0",
                    msg: distribute_rewards(
                      recipient: "referrer_new",
                      amount: "750",
                    ),
                  )),
                  reply_on: never,
                ),
              ],
              attributes: [],
              events: [],
            )"#]],
    );

    let res: DisplayResponse = exec_ok!(
        deps,
        "collector",
        ExecuteMsg::ConfigureDapp {
            dapp: "dapp".to_owned(),
            percent: None,
            collector: Some("collector_new".to_owned()),
            repo_url: None,
        }
    );

    check(
        pretty(&res),
        expect![[r#"
            (
              data: None,
              messages: [],
              attributes: [],
              events: [],
            )"#]],
    );

    let res: DisplayResponse<(), PotExecuteMsg> = exec_ok!(
        deps,
        "collector_new",
        ExecuteMsg::CollectDapp {
            dapp: "dapp".to_owned(),
        }
    );

    check(
        pretty(&res),
        expect![[r#"
            (
              data: None,
              messages: [
                (
                  id: 0,
                  msg: Wasm(Execute(
                    contract_addr: "rewards_pot_0",
                    msg: distribute_rewards(
                      recipient: "collector_new",
                      amount: "4250",
                    ),
                  )),
                  reply_on: never,
                ),
              ],
              attributes: [],
              events: [],
            )"#]],
    );
}

#[test]
fn self_referral_forwarding_works() {
    let mut deps =
        archway_bindings::testing::mock_dependencies(move |q| archway_query_handler(q, 0));

    deps.querier.update_wasm(wasm_query_handler);

    let _: DisplayResponse<(), ExecuteMsg> = init_ok!(
        deps,
        "hub_owner",
        InstantiateMsg {
            rewards_pot_code_id: 1,
        }
    );

    let _: DisplayResponse<ReferralCodeResponse> =
        exec_ok!(deps, "referrer", ExecuteMsg::RegisterReferrer {});

    let res: DisplayResponse<ReferralCodeResponse, ExecuteMsg> = exec_ok!(
        deps,
        "another_referrer",
        WithReferralCode {
            referral_code: Some(1),
            msg: ExecuteMsg::RegisterReferrer {}
        }
    );

    check(
        pretty(&res),
        expect![[r#"
            (
              data: Some((
                code: 2,
              )),
              messages: [
                (
                  id: 0,
                  msg: Wasm(Execute(
                    contract_addr: "referrals_hub",
                    msg: record_referral(
                      code: 1,
                    ),
                  )),
                  reply_on: never,
                ),
              ],
              attributes: [],
              events: [],
            )"#]],
    );
}
