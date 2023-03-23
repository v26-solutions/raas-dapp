use referrals_core::hub::dapp;

use crate::{check, expect, pretty};

#[cfg(test)]
use super::*;

#[test]
pub fn works() {
    let mut api = MockApi::default()
        .dapp("dapp")
        .collector("collector")
        .rewards_pot("rewards_pot");

    let res = dapp::deregister(
        &mut api,
        &Id::from("collector"),
        Id::from("dapp"),
        Id::from("new_admin"),
        Id::from("new_recipient"),
    )
    .unwrap();

    check(
        pretty(&res),
        expect![[r#"
            [
                WithdrawPending(Id("rewards_pot")),
                SetRewardsRecipient {
                    dapp: Id("dapp"),
                    recipient: Id("new_recipient"),
                },
                SetRewardsAdmin {
                    dapp: Id("dapp"),
                    admin: Id("new_admin"),
                },
            ]"#]],
    );

    let from_dapp_res = dapp::deregister(
        &mut MockApi::default()
            .dapp("dapp")
            .collector("collector")
            .rewards_pot("rewards_pot"),
        &Id::from("dapp"),
        Id::from("dapp"),
        Id::from("new_admin"),
        Id::from("new_recipient"),
    )
    .unwrap();

    assert_eq!(res, from_dapp_res);

    check(
        pretty(&api),
        expect![[r#"
            MockApi {
                dapp: None,
                percent: None,
                collector: Some("collector"),
                rewards_pot: Some("rewards_pot"),
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
    let mut api = MockApi::default()
        .collector("collector")
        .rewards_pot("rewards_pot");

    let res = dapp::deregister(
        &mut api,
        &Id::from("collector"),
        Id::from("dapp"),
        Id::from("new_admin"),
        Id::from("new_recipient"),
    )
    .unwrap_err();

    check(res, expect!["dapp not registered"]);
}

#[test]
pub fn sender_not_dapp_or_collector_fails() {
    let mut api = MockApi::default()
        .dapp("dapp")
        .collector("collector")
        .rewards_pot("rewards_pot");

    let res = dapp::deregister(
        &mut api,
        &Id::from("bob"),
        Id::from("dapp"),
        Id::from("new_admin"),
        Id::from("new_recipient"),
    )
    .unwrap_err();

    check(res, expect!["unauthorised"]);
}
