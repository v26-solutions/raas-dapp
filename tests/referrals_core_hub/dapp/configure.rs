use referrals_core::hub::dapp;
use referrals_core::hub::DappMetadata;

use crate::{check, expect, pretty};

use super::*;

#[test]
pub fn works() {
    let mut api = MockApi::default().dapp("dapp").collector("collector");

    dapp::configure(
        &mut api,
        &Id::from("collector"),
        &Id::from("dapp"),
        DappMetadata {
            percent: Some(nzp!(50)),
            collector: Some(Id::from("new_collector")),
            repo_url: Some("repo_url".to_owned()),
        },
    )
    .unwrap();

    dapp::configure(
        &mut MockApi::default().dapp("dapp").collector("collector"),
        &Id::from("dapp"),
        &Id::from("dapp"),
        DappMetadata {
            percent: Some(nzp!(50)),
            collector: Some(Id::from("new_collector")),
            repo_url: Some("repo_url".to_owned()),
        },
    )
    .unwrap();

    check(
        pretty(&api),
        expect![[r#"
            (
              dapp: Some(("dapp", "dapp")),
              percent: Some(50),
              collector: Some("new_collector"),
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
            )"#]],
    );
}

#[test]
pub fn not_registered_fails() {
    let mut api = MockApi::default().collector("collector");

    let res = dapp::configure(
        &mut api,
        &Id::from("dapp"),
        &Id::from("dapp"),
        DappMetadata {
            percent: Some(nzp!(50)),
            collector: Some(Id::from("new_collector")),
            repo_url: Some("repo_url".to_owned()),
        },
    )
    .unwrap_err();

    check(res, expect!["dapp not registered"]);
}

#[test]
pub fn sender_not_dapp_or_collector_fails() {
    let mut api = MockApi::default().dapp("dapp").collector("collector");

    let res = dapp::configure(
        &mut api,
        &Id::from("bob"),
        &Id::from("dapp"),
        DappMetadata {
            percent: Some(nzp!(50)),
            collector: Some(Id::from("new_collector")),
            repo_url: Some("repo_url".to_owned()),
        },
    )
    .unwrap_err();

    check(res, expect!["unauthorised"]);
}
