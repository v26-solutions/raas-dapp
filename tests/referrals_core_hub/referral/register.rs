use referrals_core::hub::referral;

use crate::{check, expect, pretty};

use super::*;

#[test]
pub fn works() {
    let mut api = MockApi::default();

    let res = referral::register(&mut api, Id::from("referrer")).unwrap();

    check(pretty(&res), expect!["Code(1)"]);

    check(
        pretty(&api),
        expect![[r#"
            MockApi {
                dapp: None,
                percent: None,
                collector: None,
                rewards_pot: None,
                rewards_pot_admin: None,
                rewards_admin: None,
                current_fee: None,
                referral_code: Some(1),
                referral_code_owner: Some("referrer"),
                latest_referral_code: Some(1),
                dapp_reffered_invocations: 0,
                code_total_earnings: 0,
                code_dapp_earnings: 0,
                dapp_contributions: 0,
                code_total_collected: 0,
                code_dapp_collected: 0,
                dapp_total_collected: 0,
                dapp_total_rewards: 0,
            }"#]],
    );
}

#[test]
pub fn already_a_referral_code_owner_fails() {
    let mut api = MockApi::default().referral_code_owner("referrer");

    let res = referral::register(&mut api, Id::from("referrer")).unwrap_err();

    check(res, expect!["already registered"]);
}
