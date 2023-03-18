use referrals_core::MutableDappStore;

use super::*;

#[test]
pub fn works() {
    let mut api = MockApi::default()
        .dapp("dapp")
        .current_fee(nz!(1000))
        .referral_code_owner("referrer")
        .referral_code(1);

    api.set_percent(&Id::from("dapp"), nzp!(50)).unwrap();

    referral::record(&mut api, &Id::from("dapp"), ReferralCode::from(1)).unwrap();

    check(
        pretty(&api),
        expect![[r#"
            MockApi {
                dapp: Some("dapp"),
                percent: Some(50),
                collector: None,
                rewards_pot: None,
                rewards_pot_admin: None,
                rewards_admin: None,
                current_fee: Some(1000),
                referral_code: Some(1),
                referral_code_owner: Some("referrer"),
                latest_referral_code: None,
                dapp_reffered_invocations: 1,
                code_total_earnings: 500,
                code_dapp_earnings: 500,
                dapp_contributions: 500,
                code_total_collected: 0,
                code_dapp_collected: 0,
                dapp_total_collected: 0,
                dapp_total_rewards: 0,
            }"#]],
    );
}

#[test]
pub fn dapp_not_registered_fails() {
    let mut api = MockApi::default()
        .referral_code_owner("referrer")
        .referral_code(1);

    let res = referral::record(&mut api, &Id::from("dapp"), ReferralCode::from(1)).unwrap_err();

    check(res, expect!["dapp not registered"]);
}

#[test]
pub fn code_not_registered_fails() {
    let mut api = MockApi::default().dapp("dapp");

    let res = referral::record(&mut api, &Id::from("dapp"), ReferralCode::from(1)).unwrap_err();

    check(res, expect!["referral code not registered"]);
}

#[test]
pub fn calculation_overflow_fails() {
    let mut api = MockApi::default()
        .dapp("dapp")
        .current_fee(NonZeroU128::new(u128::max_value()).unwrap())
        .referral_code_owner("referrer")
        .referral_code(1);

    api.set_percent(&Id::from("dapp"), nzp!(50)).unwrap();

    let res = referral::record(&mut api, &Id::from("dapp"), ReferralCode::from(1)).unwrap_err();

    check(res, expect!["math overflow"]);
}
