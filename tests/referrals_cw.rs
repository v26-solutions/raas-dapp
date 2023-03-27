use referrals_cw::{ExecuteMsg, WithReferralCode};

use serde_json_wasm::{from_str, to_string};

use crate::{check, expect};

#[test]
pub fn with_referral_code_json_serde() {
    check(
        to_string(&WithReferralCode {
            referral_code: None,
            msg: ExecuteMsg::RegisterReferrer {},
        })
        .unwrap(),
        expect![[r#"{"referral_code":null,"register_referrer":{}}"#]],
    );

    let msg: WithReferralCode<ExecuteMsg> = from_str(
        "{
            \"register_referrer\": {}
        }",
    )
    .unwrap();

    assert!(matches!(
        msg,
        WithReferralCode {
            referral_code: None,
            msg: ExecuteMsg::RegisterReferrer {}
        }
    ));

    check(
        to_string(&WithReferralCode {
            referral_code: Some(69),
            msg: ExecuteMsg::RegisterReferrer {},
        })
        .unwrap(),
        expect![[r#"{"referral_code":69,"register_referrer":{}}"#]],
    );

    let msg: WithReferralCode<ExecuteMsg> = from_str(
        "{
            \"referral_code\": 69,
            \"register_referrer\": {}
        }",
    )
    .unwrap();

    assert!(matches!(
        msg,
        WithReferralCode {
            referral_code: Some(69),
            msg: ExecuteMsg::RegisterReferrer {}
        }
    ));

    check(
        to_string(&WithReferralCode {
            referral_code: None,
            msg: ExecuteMsg::ConfigureDapp {
                dapp: "dapp".to_owned(),
                percent: Some(89),
                collector: Some("collector".to_string()),
                repo_url: Some("repo.com".to_owned()),
            },
        })
        .unwrap(),
        expect![[
            r#"{"referral_code":null,"configure_dapp":{"dapp":"dapp","percent":89,"collector":"collector","repo_url":"repo.com"}}"#
        ]],
    );

    let msg: WithReferralCode<ExecuteMsg> = from_str(
        r#"{
            "referral_code": null,
            "configure_dapp": {
                "dapp": "dapp",
                "percent": 89,
                "collector": "collector",
                "repo_url": "repo.com"
            }
        }"#,
    )
    .unwrap();

    assert!(matches!(
        msg,
        WithReferralCode {
            referral_code: None,
            msg: ExecuteMsg::ConfigureDapp {
                percent: Some(89),
                ..
            },
        }
    ));

    check(
        to_string(&WithReferralCode {
            referral_code: None,
            msg: ExecuteMsg::TransferOwnership {
                code: 69,
                owner: "owner".to_owned(),
            },
        })
        .unwrap(),
        expect![[r#"{"referral_code":null,"transfer_ownership":{"code":69,"owner":"owner"}}"#]],
    );

    let msg: WithReferralCode<ExecuteMsg> =
        from_str(r#"{"transfer_ownership":{"code":69,"owner":"owner"}}"#).unwrap();

    assert!(matches!(
        msg,
        WithReferralCode {
            referral_code: None,
            msg: ExecuteMsg::TransferOwnership { code: 69, .. },
        }
    ))
}
