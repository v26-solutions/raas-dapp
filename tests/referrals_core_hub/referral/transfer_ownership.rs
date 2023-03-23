use referrals_core::hub::referral;

use crate::{check, expect, pretty};

use super::*;

#[test]
pub fn works() {
    let mut api = MockApi::default()
        .referral_code_owner("referrer")
        .referral_code(1);

    referral::transfer_ownership(
        &mut api,
        &Id::from("referrer"),
        ReferralCode::from(1),
        Id::from("new_owner"),
    )
    .unwrap();

    check(
        pretty(&api),
        expect![[r#"
            (
              dapp: None,
              percent: None,
              collector: None,
              rewards_pot: None,
              rewards_pot_admin: None,
              rewards_admin: None,
              current_fee: None,
              referral_code: Some(1),
              referral_code_owner: Some("new_owner"),
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
pub fn code_not_registered_fails() {
    let mut api = MockApi::default()
        .referral_code_owner("referrer")
        .referral_code(1);

    let res = referral::transfer_ownership(
        &mut api,
        &Id::from("referrer"),
        ReferralCode::from(2),
        Id::from("new_owner"),
    )
    .unwrap_err();

    check(res, expect!["referral code not registered"]);
}

#[test]
pub fn sender_not_code_owner_fails() {
    let mut api = MockApi::default()
        .referral_code_owner("referrer")
        .referral_code(1);

    let res = referral::transfer_ownership(
        &mut api,
        &Id::from("bob"),
        ReferralCode::from(1),
        Id::from("new_owner"),
    )
    .unwrap_err();

    check(res, expect!["unauthorised"]);
}
