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

    assert!(msg.referral_code.is_none())
}
