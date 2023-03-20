use archway_bindings::types::rewards::{
    RewardsRecord, RewardsRecordsResponse, WithdrawRewardsResponse,
};
use archway_bindings::{testing::MockDepsExt, ArchwayQuery, PageResponse};
use cosmwasm_std::{
    coins, to_binary, Addr, ContractResult, QueryResponse, SubMsgResponse, SubMsgResult, Uint128,
};
use referrals_archway_drivers::rewards_pot;
use referrals_archway_drivers::rewards_pot::{ExecuteMsg, InstantiateMsg, QueryMsg};
use referrals_cw::rewards_pot::{
    AdminResponse, DappResponse, InstantiateResponse, TotalRewardsResponse,
};

use crate::{check, expect, pretty};

use super::DisplayResponse;

pub fn archway_query_handler(
    query: &ArchwayQuery,
    records: &[RewardsRecord],
) -> ContractResult<QueryResponse> {
    let response = match query {
        ArchwayQuery::RewardsRecords { pagination, .. } => {
            let records = pagination.clone().map_or(records.to_vec(), |p| {
                let iter = records.iter().cloned();
                let reverse = matches!(p.reverse, Some(true));
                let limit = p
                    .limit
                    .and_then(|n| (n > 0).then_some(n))
                    .map(usize::try_from)
                    .transpose()
                    .unwrap();

                match (reverse, limit) {
                    (true, None) => iter.rev().collect(),
                    (true, Some(n)) => iter.rev().take(n).collect(),

                    (false, None) => iter.collect(),
                    (false, Some(n)) => iter.take(n).collect(),
                }
            });

            let len = records.len();

            to_binary(&RewardsRecordsResponse {
                records,
                pagination: Some(PageResponse {
                    next_key: None,
                    total: Some(len as _),
                }),
            })
        }

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
        env.contract.address = Addr::unchecked("rewards_pot");
        env
    }};
}

macro_rules! _do {
    ($op:ident, $deps:ident, $from:expr, $msg:expr) => {{
        rewards_pot::$op(&mut $deps.as_mut(), &env!(), $from, $msg)
    }};
}

macro_rules! init_ok {
    ($deps:ident, $from:literal, $msg:expr) => {
        _do!(init, $deps, info!($from), $msg)
            .map(From::from)
            .unwrap()
    };
}

macro_rules! exec_ok {
    ($deps:ident, $from:literal, $msg:expr) => {
        _do!(execute, $deps, &info!($from), $msg)
            .map(From::from)
            .unwrap()
    };
}

macro_rules! exec_err {
    ($deps:ident, $from:literal, $msg:expr) => {
        _do!(execute, $deps, &info!($from), $msg).unwrap_err()
    };
}

macro_rules! _reply {
    ($deps:ident, $msg:expr) => {{
        let reply = cosmwasm_std::Reply {
            id: 0,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(cosmwasm_std::to_binary(&$msg).unwrap()),
            }),
        };
        rewards_pot::reply(&mut $deps.as_mut(), &env!(), reply).map(DisplayResponse::from)
    }};
}

macro_rules! reply_ok {
    ($deps:ident, $msg:expr) => {{
        _reply!($deps, $msg).unwrap()
    }};
}

macro_rules! query_ok {
    ($deps:ident, $msg:expr) => {{
        let bin = rewards_pot::query(&$deps.as_ref(), &env!(), &$msg).unwrap();
        cosmwasm_std::from_binary(&bin).unwrap()
    }};
}

#[test]
fn plumbing_works() {
    let records = vec![
        RewardsRecord {
            id: 1,
            rewards_address: String::from("rewards_pot"),
            rewards: coins(1000, "ucosm"),
            calculated_height: 12345,
            calculated_time: String::from("2022-11-11T11:11:22"),
        },
        RewardsRecord {
            id: 2,
            rewards_address: String::from("rewards_pot"),
            rewards: coins(1000, "cosm"),
            calculated_height: 12346,
            calculated_time: String::from("2022-11-11T11:22:33"),
        },
        RewardsRecord {
            id: 3,
            rewards_address: String::from("rewards_pot"),
            rewards: coins(1000, "cosm"),
            calculated_height: 12346,
            calculated_time: String::from("2022-11-11T11:22:33"),
        },
    ];

    let mut deps =
        archway_bindings::testing::mock_dependencies(move |q| archway_query_handler(q, &records));

    let res: DisplayResponse<InstantiateResponse> = init_ok!(
        deps,
        "referrals_hub",
        InstantiateMsg {
            dapp: "dapp".to_owned()
        }
    );

    check(
        pretty(&res),
        expect![[r#"
            Response {
                data: Some(InstantiateResponse {
                    dapp: "dapp",
                }),
                messages: [],
            }"#]],
    );

    let res: DappResponse = query_ok!(deps, QueryMsg::Dapp {});

    check(pretty(&res), expect![[r#"DappResponse { dapp: "dapp" }"#]]);

    let res: AdminResponse = query_ok!(deps, QueryMsg::Admin {});

    check(
        pretty(&res),
        expect![[r#"
        AdminResponse {
            admin: "referrals_hub",
        }"#]],
    );

    let res: TotalRewardsResponse = query_ok!(deps, QueryMsg::TotalRewards {});

    check(
        pretty(&res),
        expect![[r#"
            TotalRewardsResponse {
                total: 3000,
            }"#]],
    );

    let res: DisplayResponse = exec_ok!(deps, "referrals_hub", ExecuteMsg::WithdrawRewards {});

    check(
        pretty(&res),
        expect![[r#"
            Response {
                data: None,
                messages: [
                    WithdrawRewards {
                        records_limit: Some(3),
                        record_ids: [],
                        reply_on: "Success",
                    },
                ],
            }"#]],
    );

    let res: DisplayResponse = reply_ok!(
        deps,
        WithdrawRewardsResponse {
            records_num: 3,
            total_rewards: cosmwasm_std::coins(3000, "ucosm")
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

    let res: DisplayResponse = exec_ok!(
        deps,
        "referrals_hub",
        ExecuteMsg::DistributeRewards {
            recipient: "collector".to_owned(),
            amount: Uint128::new(1000),
        }
    );

    check(
        pretty(&res),
        expect![[r#"
            Response {
                data: None,
                messages: [
                    BankSend {
                        to_address: "collector",
                        amount: Some("1000 ucosm"),
                        reply_on: "Never",
                    },
                ],
            }"#]],
    );

    // two more records added since collection

    let records = vec![
        RewardsRecord {
            id: 1,
            rewards_address: String::from("rewards_pot"),
            rewards: coins(1000, "ucosm"),
            calculated_height: 12345,
            calculated_time: String::from("2022-11-11T11:11:22"),
        },
        RewardsRecord {
            id: 2,
            rewards_address: String::from("rewards_pot"),
            rewards: coins(1000, "cosm"),
            calculated_height: 12346,
            calculated_time: String::from("2022-11-11T11:22:33"),
        },
        RewardsRecord {
            id: 3,
            rewards_address: String::from("rewards_pot"),
            rewards: coins(1000, "cosm"),
            calculated_height: 12346,
            calculated_time: String::from("2022-11-11T11:22:33"),
        },
        RewardsRecord {
            id: 4,
            rewards_address: String::from("rewards_pot"),
            rewards: coins(1000, "cosm"),
            calculated_height: 12346,
            calculated_time: String::from("2022-11-11T11:22:33"),
        },
        RewardsRecord {
            id: 5,
            rewards_address: String::from("rewards_pot"),
            rewards: coins(1000, "cosm"),
            calculated_height: 12346,
            calculated_time: String::from("2022-11-11T11:22:33"),
        },
    ];

    let deps = deps.with_archway_query_handler(move |q| archway_query_handler(q, &records));

    let res: TotalRewardsResponse = query_ok!(deps, QueryMsg::TotalRewards {});

    check(
        pretty(&res),
        expect![[r#"
            TotalRewardsResponse {
                total: 5000,
            }"#]],
    );
}

#[test]
fn non_admin_exec_fails() {
    let mut deps =
        archway_bindings::testing::mock_dependencies(move |q| archway_query_handler(q, &[]));

    let _: DisplayResponse<InstantiateResponse> = init_ok!(
        deps,
        "referrals_hub",
        InstantiateMsg {
            dapp: "dapp".to_owned()
        }
    );

    let res = exec_err!(deps, "bob", ExecuteMsg::WithdrawRewards {});

    check(res, expect!["unauthorized"]);

    let res = exec_err!(
        deps,
        "bob",
        ExecuteMsg::DistributeRewards {
            recipient: "collector".to_owned(),
            amount: Uint128::new(1000),
        }
    );

    check(res, expect!["unauthorized"]);
}
