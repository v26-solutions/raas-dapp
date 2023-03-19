use super::*;

#[test]
pub fn works() {
    let mut api = MockApi::default().dapp("dapp").collector("collector");

    let res = dapp::set_fee(
        &mut api,
        &Id::from("collector"),
        Id::from("dapp"),
        nz!(1000),
    )
    .unwrap();

    check(
        pretty(&res),
        expect![[r#"
            SetDappFee {
                dapp: Id("dapp"),
                amount: 1000,
            }"#]],
    );

    let from_dapp_res =
        dapp::set_fee(&mut api, &Id::from("dapp"), Id::from("dapp"), nz!(1000)).unwrap();

    assert_eq!(res, from_dapp_res);

    check(
        pretty(&api),
        expect![[r#"
            MockApi {
                dapp: Some(("dapp", "dapp")),
                percent: None,
                collector: Some("collector"),
                rewards_pot: None,
                rewards_pot_admin: None,
                rewards_admin: None,
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
    let mut api = MockApi::default().collector("collector");

    let res = dapp::set_fee(
        &mut api,
        &Id::from("collector"),
        Id::from("dapp"),
        nz!(1000),
    )
    .unwrap_err();

    check(res, expect!["dapp not registered"]);
}

#[test]
pub fn sender_not_dapp_or_collector_fails() {
    let mut api = MockApi::default().dapp("dapp").collector("collector");

    let res = dapp::set_fee(&mut api, &Id::from("bob"), Id::from("dapp"), nz!(1000)).unwrap_err();

    check(res, expect!["unauthorised"]);
}
