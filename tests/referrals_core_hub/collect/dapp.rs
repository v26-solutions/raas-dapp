use referrals_core::hub::collect;
use referrals_core::hub::MutableReferralStore;

use crate::{check, expect, pretty};

use super::*;

#[test]
fn works() {
    let mut api = MockApi::default()
        .dapp("dapp")
        .rewards_pot("rewards_pot")
        .collector("collector")
        .referral_code(1)
        .dapp_total_rewards(11_000);

    api.set_dapp_contributions(&Id::from("dapp"), nz!(5000))
        .unwrap();

    let res = collect::dapp(&mut api, Id::from("collector"), &Id::from("dapp")).unwrap();

    check(
        pretty(&res),
        expect![[r#"
            RedistributeRewards(
              amount: 6000,
              pot: ("rewards_pot"),
              receiver: ("collector"),
            )"#]],
    );

    check(
        pretty(&api),
        expect![[r#"
            (
              dapp: Some(("dapp", "dapp")),
              percent: None,
              collector: Some("collector"),
              rewards_pot: Some("rewards_pot"),
              rewards_pot_admin: None,
              rewards_admin: None,
              current_fee: None,
              referral_code: Some(1),
              referral_code_owner: None,
              latest_referral_code: None,
              dapp_reffered_invocations: 0,
              code_total_earnings: 0,
              code_dapp_earnings: 0,
              dapp_contributions: 5000,
              code_total_collected: 0,
              code_dapp_collected: 0,
              dapp_total_collected: 6000,
              dapp_total_rewards: 11000,
            )"#]],
    );

    api.set_dapp_contributions(&Id::from("dapp"), nz!(10_000))
        .unwrap();

    api.set_dapp_total_rewards(22_000);

    let res = collect::dapp(&mut api, Id::from("dapp"), &Id::from("dapp")).unwrap();

    check(
        pretty(&res),
        expect![[r#"
            RedistributeRewards(
              amount: 6000,
              pot: ("rewards_pot"),
              receiver: ("dapp"),
            )"#]],
    );

    check(
        pretty(&api),
        expect![[r#"
            (
              dapp: Some(("dapp", "dapp")),
              percent: None,
              collector: Some("collector"),
              rewards_pot: Some("rewards_pot"),
              rewards_pot_admin: None,
              rewards_admin: None,
              current_fee: None,
              referral_code: Some(1),
              referral_code_owner: None,
              latest_referral_code: None,
              dapp_reffered_invocations: 0,
              code_total_earnings: 0,
              code_dapp_earnings: 0,
              dapp_contributions: 10000,
              code_total_collected: 0,
              code_dapp_collected: 0,
              dapp_total_collected: 12000,
              dapp_total_rewards: 22000,
            )"#]],
    );
}

#[test]
fn sender_not_dapp_or_collector_fails() {
    let mut api = MockApi::default().dapp("dapp").collector("collector");

    let res = collect::dapp(&mut api, Id::from("bob"), &Id::from("dapp")).unwrap_err();

    check(res, expect!["unauthorised"]);
}

#[test]
fn no_earnings_to_collect_fails() {
    let mut api = MockApi::default()
        .dapp("dapp")
        .rewards_pot("rewards_pot")
        .collector("collector")
        .referral_code(1);

    let res = collect::dapp(&mut api, Id::from("collector"), &Id::from("dapp")).unwrap_err();

    check(res, expect!["nothing to collect"]);

    api.set_dapp_contributions(&Id::from("dapp"), nz!(5000))
        .unwrap();

    api.set_dapp_total_rewards(5000);

    let res = collect::dapp(&mut api, Id::from("collector"), &Id::from("dapp")).unwrap_err();

    check(res, expect!["nothing to collect"]);

    api.set_dapp_contributions(&Id::from("dapp"), nz!(5000))
        .unwrap();

    api.set_dapp_total_rewards(11_000);

    collect::dapp(&mut api, Id::from("collector"), &Id::from("dapp")).unwrap();

    let res = collect::dapp(&mut api, Id::from("collector"), &Id::from("dapp")).unwrap_err();

    check(res, expect!["nothing to collect"]);
}
