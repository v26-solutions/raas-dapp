use super::*;

#[test]
pub fn works() {
    let mut api = MockApi::default().rewards_admin(SELF_ID);

    let res = dapp::register(
        &mut api,
        Id::from("dapp"),
        "dapp".to_owned(),
        nzp!(100),
        Id::from("collector"),
    )
    .unwrap();

    check(debug(res), expect![[r#"CreateRewardsPot(Id("dapp"))"#]]);

    check(
        pretty(&api),
        expect![[r#"
            MockApi {
                dapp: Some(("dapp", "dapp")),
                percent: Some(100),
                collector: Some("collector"),
                rewards_pot: None,
                rewards_pot_admin: None,
                rewards_admin: Some("self"),
                current_fee: None,
                referral_code: None,
                referral_code_owner: None,
                latest_referral_code: None,
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
pub fn already_registered_fails() {
    let mut api = MockApi::default().dapp("dapp");

    let res = dapp::register(
        &mut api,
        Id::from("dapp"),
        "dapp".to_owned(),
        nzp!(100),
        Id::from("collector"),
    )
    .unwrap_err();

    check(res, expect!["already registered"]);
}

#[test]
pub fn not_referrals_admin_fails() {
    let mut api = MockApi::default().rewards_admin("bob");

    let res = dapp::register(
        &mut api,
        Id::from("dapp"),
        "dapp".to_owned(),
        nzp!(100),
        Id::from("collector"),
    )
    .unwrap_err();

    check(res, expect!["invalid rewards admin"]);
}
