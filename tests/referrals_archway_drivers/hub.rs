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
            Response {
                data: None,
                messages: [
                    UpdateContractMetadata {
                        contract_address: None,
                        owner_address: Some("referrals_hub"),
                        rewards_address: None,
                        reply_on: "Never",
                    },
                    WasmExecute {
                        contract_addr: "referrals_hub",
                        msg: RegisterDapp {
                            name: "referrals_hub",
                            percent: 100,
                            collector: "hub_owner",
                        },
                        funds: None,
                        reply_on: "Never",
                    },
                ],
            }"#]],
    );

    let res: DisplayResponse<ReferralCodeResponse> =
        exec_ok!(deps, "referrer", ExecuteMsg::RegisterReferrer {});

    check(
        pretty(&res),
        expect![[r#"
                    Response {
                        data: Some(ReferralCodeResponse { code: 1 }),
                        messages: [],
                    }"#]],
    );

    let res: DisplayResponse<(), PotInitMsg> = exec_ok!(
        deps,
        "dapp",
        ExecuteMsg::RegisterDapp {
            name: "dapp".to_owned(),
            percent: 75,
            collector: "collector".to_owned(),
        }
    );

    check(
        pretty(&res),
        expect![[r#"
                    Response {
                        data: None,
                        messages: [
                            WasmInstantiate {
                                admin: None,
                                code_id: 1,
                                msg: InstantiateMsg { dapp: "dapp" },
                                funds: None,
                                label: "referrals-reward-pot-0",
                                reply_on: "Success",
                            },
                        ],
                    }"#]],
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
            Response {
                data: None,
                messages: [
                    SetFlatFee {
                        contract_address: Some("dapp"),
                        flat_fee_amount: 1000,
                        flat_fee_denom: "",
                        reply_on: "Never",
                    },
                ],
            }"#]],
    );

    let mut deps = deps.with_archway_query_handler(move |q| archway_query_handler(q, 1000));

    let res: DisplayResponse = exec_ok!(deps, "dapp", ExecuteMsg::RecordReferral { code: 1 });

    check(
        pretty(&res),
        expect![[r#"
                    Response {
                        data: None,
                        messages: [],
                    }"#]],
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
                    Response {
                        data: None,
                        messages: [],
                    }"#]],
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
            Response {
                data: None,
                messages: [
                    WasmExecute {
                        contract_addr: "rewards_pot_0",
                        msg: DistibuteRewards {
                            recipient: "referrer_new",
                            amount: 750,
                        },
                        funds: None,
                        reply_on: "Never",
                    },
                ],
            }"#]],
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
                    Response {
                        data: None,
                        messages: [],
                    }"#]],
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
            Response {
                data: None,
                messages: [
                    WasmExecute {
                        contract_addr: "rewards_pot_0",
                        msg: DistibuteRewards {
                            recipient: "collector_new",
                            amount: 4250,
                        },
                        funds: None,
                        reply_on: "Never",
                    },
                ],
            }"#]],
    );
}

#[test]
fn self_referral_forwarding_works() {
    let mut deps =
        archway_bindings::testing::mock_dependencies(move |q| archway_query_handler(q, 0));

    deps.querier.update_wasm(wasm_query_handler);

    let _: DisplayResponse<ExecuteMsg> = init_ok!(
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
            Response {
                data: Some(ReferralCodeResponse { code: 2 }),
                messages: [
                    WasmExecute {
                        contract_addr: "referrals_hub",
                        msg: RecordReferral { code: 1 },
                        funds: None,
                        reply_on: "Never",
                    },
                ],
            }"#]],
    );
}
