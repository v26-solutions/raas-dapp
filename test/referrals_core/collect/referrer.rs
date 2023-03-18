use referrals_core::DappStore;

use super::*;

#[test]
fn works() {
    let mut api = MockApi::default()
        .dapp("dapp")
        .rewards_pot("rewards_pot")
        .referral_code(1)
        .referral_code_owner("referrer")
        .dapp_total_rewards(11_000);

    api.set_total_earnings(ReferralCode::from(1), nz!(5000))
        .unwrap();

    api.set_dapp_earnings(&Id::from("dapp"), ReferralCode::from(1), nz!(5000))
        .unwrap();

    let res = collect::referrer(
        &mut api,
        Id::from("referrer"),
        &Id::from("dapp"),
        ReferralCode::from(1),
    )
    .unwrap();

    check(
        debug_slice(&res),
        expect![[r#"
            [
            	WithdrawPending(Id("rewards_pot"))
            	RedistributeRewards { amount: 5000, pot: Id("rewards_pot"), receiver: Id("referrer") }
            ]
        "#]],
    );

    check(
        pretty(&api),
        expect![[r#"
                MockApi {
                    dapp: Some("dapp"),
                    percent: None,
                    collector: None,
                    rewards_pot: Some("rewards_pot"),
                    rewards_pot_admin: None,
                    rewards_admin: None,
                    current_fee: None,
                    referral_code: Some(1),
                    referral_code_owner: Some("referrer"),
                    latest_referral_code: None,
                    dapp_reffered_invocations: 0,
                    code_total_earnings: 5000,
                    code_dapp_earnings: 5000,
                    dapp_contributions: 0,
                    code_total_collected: 5000,
                    code_dapp_collected: 5000,
                    dapp_total_collected: 0,
                    dapp_total_rewards: 11000,
                }"#]],
    );

    api.set_total_earnings(ReferralCode::from(1), nz!(7000))
        .unwrap();

    api.set_dapp_earnings(&Id::from("dapp"), ReferralCode::from(1), nz!(7000))
        .unwrap();

    let res = collect::referrer(
        &mut api,
        Id::from("referrer"),
        &Id::from("dapp"),
        ReferralCode::from(1),
    )
    .unwrap();

    check(
        debug_slice(&res),
        expect![[r#"
                [
                	WithdrawPending(Id("rewards_pot"))
                	RedistributeRewards { amount: 2000, pot: Id("rewards_pot"), receiver: Id("referrer") }
                ]
            "#]],
    );

    check(
        pretty(&api),
        expect![[r#"
                MockApi {
                    dapp: Some("dapp"),
                    percent: None,
                    collector: None,
                    rewards_pot: Some("rewards_pot"),
                    rewards_pot_admin: None,
                    rewards_admin: None,
                    current_fee: None,
                    referral_code: Some(1),
                    referral_code_owner: Some("referrer"),
                    latest_referral_code: None,
                    dapp_reffered_invocations: 0,
                    code_total_earnings: 7000,
                    code_dapp_earnings: 7000,
                    dapp_contributions: 0,
                    code_total_collected: 7000,
                    code_dapp_collected: 7000,
                    dapp_total_collected: 0,
                    dapp_total_rewards: 11000,
                }"#]],
    );
}

#[test]
fn code_not_registered_fails() {
    let mut api = MockApi::default()
        .dapp("dapp")
        .rewards_pot("rewards_pot")
        .referral_code_owner("referrer")
        .current_fee(nz!(1000))
        .dapp_total_rewards(11_000);

    let res = collect::referrer(
        &mut api,
        Id::from("referrer"),
        &Id::from("dapp"),
        ReferralCode::from(1),
    )
    .unwrap_err();

    check(res, expect!["referral code not registered"]);
}

#[test]
fn sender_not_code_owner_fails() {
    let mut api = MockApi::default()
        .dapp("dapp")
        .referral_code(1)
        .rewards_pot("rewards_pot")
        .referral_code_owner("referrer")
        .current_fee(nz!(1000))
        .dapp_total_rewards(11_000);

    let res = collect::referrer(
        &mut api,
        Id::from("bob"),
        &Id::from("dapp"),
        ReferralCode::from(1),
    )
    .unwrap_err();

    check(res, expect!["unauthorised"]);
}

#[test]
fn no_earnings_to_collect_fails() {
    let mut api = MockApi::default()
        .dapp("dapp")
        .referral_code(1)
        .rewards_pot("rewards_pot")
        .referral_code_owner("referrer")
        .current_fee(nz!(1000))
        .dapp_total_rewards(11_000);

    let res = collect::referrer(
        &mut api,
        Id::from("referrer"),
        &Id::from("dapp"),
        ReferralCode::from(1),
    )
    .unwrap_err();

    check(res, expect!["nothing to collect"]);

    api.set_percent(&Id::from("dapp"), nzp!(50)).unwrap();

    api.set_total_earnings(ReferralCode::from(1), nz!(5000))
        .unwrap();

    api.set_dapp_earnings(&Id::from("dapp"), ReferralCode::from(1), nz!(5000))
        .unwrap();

    collect::referrer(
        &mut api,
        Id::from("referrer"),
        &Id::from("dapp"),
        ReferralCode::from(1),
    )
    .unwrap();

    let res = collect::referrer(
        &mut api,
        Id::from("referrer"),
        &Id::from("dapp"),
        ReferralCode::from(1),
    )
    .unwrap_err();

    check(res, expect!["nothing to collect"]);
}
