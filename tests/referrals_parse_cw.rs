use cosmwasm_std::{testing::MockApi, Addr, MessageInfo};
use referrals_cw::ExecuteMsg;
use referrals_parse_cw::parse_hub_exec;

use crate::{check, expect, pretty};

#[test]
fn register_referrer() {
    let mock_api = MockApi::default();
    let msg_info = MessageInfo {
        sender: Addr::unchecked("sender"),
        funds: vec![],
    };

    let res = parse_hub_exec(&mock_api, msg_info, ExecuteMsg::RegisterReferrer {}).unwrap();

    check(
        pretty(&res),
        expect![[r#"
            Msg {
                sender: Id("sender"),
                kind: Register(Referrer),
            }"#]],
    );
}

mod register_dapp {
    use super::*;

    #[test]
    fn works() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::RegisterDapp {
                name: "dapp".to_owned(),
                percent: 100,
                collector: "collector".to_owned(),
            },
        )
        .unwrap();

        check(
            pretty(&res),
            expect![[r#"
                Msg {
                    sender: Id("sender"),
                    kind: Register(Dapp {
                        name: "dapp",
                        percent: NonZeroPercent(100),
                        collector: Id("collector"),
                    }),
                }"#]],
        );
    }

    #[test]
    fn invalid_percent_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info.clone(),
            ExecuteMsg::RegisterDapp {
                name: "dapp".to_owned(),
                percent: 101,
                collector: "collector".to_owned(),
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid percent - valid value is any integer between 1 & 100"],
        );

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::RegisterDapp {
                name: "dapp".to_owned(),
                percent: 0,
                collector: "collector".to_owned(),
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid percent - valid value is any integer between 1 & 100"],
        );
    }

    #[test]
    fn invalid_collector_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::RegisterDapp {
                name: "dapp".to_owned(),
                percent: 100,
                collector: "0".to_owned(),
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid address - Generic error: Invalid input: human address too short for this mock implementation (must be >= 3)."],
        );
    }
}

mod deregister_dapp {
    use super::*;

    #[test]
    fn works() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::DeregisterDapp {
                dapp: "dapp".to_owned(),
                rewards_admin: "rewards_admin".to_owned(),
                rewards_recipient: "new_recipient".to_owned(),
            },
        )
        .unwrap();

        check(
            pretty(&res),
            expect![[r#"
                Msg {
                    sender: Id("sender"),
                    kind: Register(DeregisterDapp {
                        dapp: Id("dapp"),
                        rewards_admin: Id("rewards_admin"),
                        rewards_recipient: Id("new_recipient"),
                    }),
                }"#]],
        );
    }

    #[test]
    fn invalid_dapp_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::DeregisterDapp {
                dapp: "0".to_owned(),
                rewards_admin: "rewards_admin".to_owned(),
                rewards_recipient: "new_recipient".to_owned(),
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid address - Generic error: Invalid input: human address too short for this mock implementation (must be >= 3)."],
        );
    }

    #[test]
    fn invalid_rewards_admin_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::DeregisterDapp {
                dapp: "dapp".to_owned(),
                rewards_admin: "0".to_owned(),
                rewards_recipient: "new_recipient".to_owned(),
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid address - Generic error: Invalid input: human address too short for this mock implementation (must be >= 3)."],
        );
    }

    #[test]
    fn invalid_rewards_recipient_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::DeregisterDapp {
                dapp: "dapp".to_owned(),
                rewards_admin: "new_admin".to_owned(),
                rewards_recipient: "0".to_owned(),
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid address - Generic error: Invalid input: human address too short for this mock implementation (must be >= 3)."],
        );
    }
}

mod set_dapp_fee {
    use cosmwasm_std::Uint128;

    use super::*;

    #[test]
    fn works() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::SetDappFee {
                dapp: "dapp".to_owned(),
                fee: Uint128::new(1000),
            },
        )
        .unwrap();

        check(
            pretty(&res),
            expect![[r#"
                Msg {
                    sender: Id("sender"),
                    kind: Config(DappFee {
                        dapp: Id("dapp"),
                        fee: 1000,
                    }),
                }"#]],
        );
    }

    #[test]
    fn invalid_dapp_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::SetDappFee {
                dapp: "0".to_owned(),
                fee: Uint128::new(1000),
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid address - Generic error: Invalid input: human address too short for this mock implementation (must be >= 3)."],
        );
    }

    #[test]
    fn invalid_fee_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::SetDappFee {
                dapp: "dapp".to_owned(),
                fee: Uint128::new(0),
            },
        )
        .unwrap_err();

        check(res, expect!["invalid fee - expected non-zero value"]);
    }
}

#[test]
fn record_referral() {
    let mock_api = MockApi::default();
    let msg_info = MessageInfo {
        sender: Addr::unchecked("sender"),
        funds: vec![],
    };

    let res = parse_hub_exec(&mock_api, msg_info, ExecuteMsg::RecordReferral { code: 1 }).unwrap();

    check(
        pretty(&res),
        expect![[r#"
            Msg {
                sender: Id("sender"),
                kind: Referral { code: Code(1) },
            }"#]],
    );
}

mod collect_referrer {
    use super::*;

    #[test]
    fn works() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::CollectReferrer {
                code: 1,
                dapp: "dapp".to_owned(),
            },
        )
        .unwrap();

        check(
            pretty(&res),
            expect![[r#"
                Msg {
                    sender: Id("sender"),
                    kind: Collect(Referrer {
                        dapp: Id("dapp"),
                        code: Code(1),
                    }),
                }"#]],
        );
    }

    #[test]
    fn invalid_dapp_address_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::CollectReferrer {
                code: 1,
                dapp: "0".to_owned(),
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid address - Generic error: Invalid input: human address too short for this mock implementation (must be >= 3)."],
        );
    }
}

mod collect_dapp {
    use super::*;

    #[test]
    fn works() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::CollectDapp {
                dapp: "dapp".to_owned(),
            },
        )
        .unwrap();

        check(
            pretty(&res),
            expect![[r#"
                Msg {
                    sender: Id("sender"),
                    kind: Collect(Dapp { dapp: Id("dapp") }),
                }"#]],
        );
    }

    #[test]
    fn invalid_dapp_address_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::CollectDapp {
                dapp: "0".to_owned(),
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid address - Generic error: Invalid input: human address too short for this mock implementation (must be >= 3)."],
        );
    }
}

mod transfer_ownership {
    use super::*;

    #[test]
    fn works() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::TransferOwnership {
                code: 1,
                owner: "new_owner".to_owned(),
            },
        )
        .unwrap();

        check(
            pretty(&res),
            expect![[r#"
                Msg {
                    sender: Id("sender"),
                    kind: Config(TransferReferralCodeOwnership {
                        code: Code(1),
                        owner: Id("new_owner"),
                    }),
                }"#]],
        );
    }

    #[test]
    fn invalid_owner_address_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::TransferOwnership {
                code: 1,
                owner: "0".to_owned(),
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid address - Generic error: Invalid input: human address too short for this mock implementation (must be >= 3)."],
        );
    }
}

mod configure_dapp {
    use super::*;

    #[test]
    fn works() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::ConfigureDapp {
                dapp: "dapp".to_owned(),
                percent: Some(100),
                collector: Some("new_collector".to_owned()),
                repo_url: None,
            },
        )
        .unwrap();

        check(
            pretty(&res),
            expect![[r#"
                Msg {
                    sender: Id("sender"),
                    kind: Config(DappMetadata {
                        dapp: Id("dapp"),
                        metadata: Metadata {
                            percent: Some(NonZeroPercent(100)),
                            collector: Some(Id("new_collector")),
                            repo_url: None,
                        },
                    }),
                }"#]],
        );
    }

    #[test]
    fn invalid_dapp_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::ConfigureDapp {
                dapp: "0".to_owned(),
                percent: Some(100),
                collector: Some("new_collector".to_owned()),
                repo_url: None,
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid address - Generic error: Invalid input: human address too short for this mock implementation (must be >= 3)."],
        );
    }

    #[test]
    fn invalid_percent_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info.clone(),
            ExecuteMsg::ConfigureDapp {
                dapp: "dapp".to_owned(),
                percent: Some(0),
                collector: Some("new_collector".to_owned()),
                repo_url: None,
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid percent - valid value is any integer between 1 & 100"],
        );

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::ConfigureDapp {
                dapp: "dapp".to_owned(),
                percent: Some(101),
                collector: Some("new_collector".to_owned()),
                repo_url: None,
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid percent - valid value is any integer between 1 & 100"],
        );
    }

    #[test]
    fn invalid_collector_fails() {
        let mock_api = MockApi::default();
        let msg_info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![],
        };

        let res = parse_hub_exec(
            &mock_api,
            msg_info,
            ExecuteMsg::ConfigureDapp {
                dapp: "dapp".to_owned(),
                percent: Some(100),
                collector: Some("0".to_owned()),
                repo_url: None,
            },
        )
        .unwrap_err();

        check(
            res,
            expect!["invalid address - Generic error: Invalid input: human address too short for this mock implementation (must be >= 3)."],
        );
    }
}
