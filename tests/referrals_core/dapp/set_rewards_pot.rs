use super::*;

#[test]
pub fn works() {
    let mut api = MockApi::default().dapp("dapp").rewards_admin(SELF_ID);

    let res = dapp::set_rewards_pot(&mut api, &Id::from("dapp"), Id::from("rewards_pot")).unwrap();

    check(
        pretty(&res),
        expect![[r#"SetRewardsRecipient(Id("rewards_pot"))"#]],
    );

    check(
        pretty(&api),
        expect![[r#"
            MockApi {
                dapp: Some(("dapp", "dapp")),
                percent: None,
                collector: None,
                rewards_pot: Some("rewards_pot"),
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
pub fn not_registered_fails() {
    let mut api = MockApi::default();

    let res =
        dapp::set_rewards_pot(&mut api, &Id::from("dapp"), Id::from("rewards_pot")).unwrap_err();

    check(res, expect!["dapp not registered"]);
}

#[test]
pub fn rewards_pot_already_set_fails() {
    let mut api = MockApi::default().dapp("dapp").rewards_pot("other_pot");

    let res =
        dapp::set_rewards_pot(&mut api, &Id::from("dapp"), Id::from("rewards_pot")).unwrap_err();

    check(res, expect!["rewards pot already set"]);
}

#[test]
pub fn not_rewards_pot_admin_fails() {
    let mut api = MockApi::default().dapp("dapp").rewards_pot_admin("bob");

    let res =
        dapp::set_rewards_pot(&mut api, &Id::from("dapp"), Id::from("rewards_pot")).unwrap_err();

    check(res, expect!["invalid rewards pot admin"]);
}
