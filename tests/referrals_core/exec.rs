use referrals_core::{
    exec, Collection, Configure, DappMetadata, Id, Msg, MsgKind, ReferralCode, Registration, Reply,
};

use crate::{check, expect, pretty};

use super::*;

struct DisplayReply(Reply);

macro_rules! exec_msg_ok {
    ($api:ident, $sender:literal, $kind:expr) => {
        exec(
            &mut $api,
            Msg {
                sender: Id::from($sender),
                kind: $kind.into(),
            },
        )
        .map(DisplayReply)
        .unwrap()
    };
}

#[test]
fn collect_earnings_post_deregister() {
    let mut api = MockApi::default()
        .rewards_admin(dapp::SELF_ID)
        .current_fee(nz!(1000));

    exec_msg_ok!(
        api,
        "dapp",
        Registration::Dapp {
            name: "dapp".to_owned(),
            percent: nzp!(50),
            collector: Id::from("collector"),
        }
    );

    exec_msg_ok!(
        api,
        "self",
        Registration::RewardsPot {
            dapp: Id::from("dapp"),
            rewards_pot: Id::from("rewards_pot")
        }
    );

    exec_msg_ok!(api, "referrer", Registration::Referrer);

    exec_msg_ok!(
        api,
        "collector",
        Configure::DappFee {
            dapp: Id::from("dapp"),
            fee: nz!(1000)
        }
    );

    exec_msg_ok!(
        api,
        "dapp",
        MsgKind::Referral {
            code: ReferralCode::from(1),
        }
    );

    api.set_dapp_total_rewards(1222);

    exec_msg_ok!(
        api,
        "collector",
        Registration::DeregisterDapp {
            dapp: Id::from("dapp"),
            rewards_admin: Id::from("collector"),
            rewards_recipient: Id::from("collector")
        }
    );

    let res = exec_msg_ok!(
        api,
        "referrer",
        Collection::Referrer {
            code: ReferralCode::from(1),
            dapp: Id::from("dapp")
        }
    );

    check(
        res,
        expect![[r#"
            [
                WithdrawPending(Id("rewards_pot")),
                RedistributeRewards {
                    amount: 500,
                    pot: Id("rewards_pot"),
                    receiver: Id("referrer"),
                },
            ]"#]],
    );

    let res = exec_msg_ok!(
        api,
        "collector",
        Collection::Dapp {
            dapp: Id::from("dapp")
        }
    );

    check(
        res,
        expect![[r#"
            [
                WithdrawPending(Id("rewards_pot")),
                RedistributeRewards {
                    amount: 722,
                    pot: Id("rewards_pot"),
                    receiver: Id("collector"),
                },
            ]"#]],
    );
}

#[test]
fn msg_routing() {
    let mut api = MockApi::default().rewards_admin(dapp::SELF_ID);

    let res = exec_msg_ok!(
        api,
        "dapp",
        Registration::Dapp {
            name: "dapp".to_owned(),
            percent: nzp!(50),
            collector: Id::from("collector"),
        }
    );

    check(res, expect![[r#"CreateRewardsPot(Id("dapp"))"#]]);

    let res = exec_msg_ok!(
        api,
        "self",
        Registration::RewardsPot {
            dapp: Id::from("dapp"),
            rewards_pot: Id::from("rewards_pot")
        }
    );

    check(res, expect![[r#"SetRewardsRecipient(Id("rewards_pot"))"#]]);

    let res = exec_msg_ok!(api, "referrer1", Registration::Referrer);

    check(res, expect!["{ code: 1 }"]);

    let res = exec_msg_ok!(
        api,
        "collector",
        Configure::DappFee {
            dapp: Id::from("dapp"),
            fee: nz!(1000)
        }
    );

    check(
        res,
        expect![[r#"SetDappFee { dapp: Id("dapp"), amount: 1000 }"#]],
    );

    api.set_current_fee(nz!(1000));

    assert_eq!(api.percent, Some(50));

    let res = exec_msg_ok!(
        api,
        "collector",
        Configure::DappMetadata {
            dapp: Id::from("dapp"),
            metadata: DappMetadata {
                percent: Some(nzp!(75)),
                collector: None,
                repo_url: Some("some_repo".to_owned()),
            }
        }
    );

    check(res, expect!["empty"]);

    assert_eq!(api.percent, Some(75));

    assert_eq!(api.referral_code_owner, Some("referrer1".to_owned()));

    let res = exec_msg_ok!(
        api,
        "referrer1",
        Configure::TransferReferralCodeOwnership {
            code: ReferralCode::from(1),
            owner: Id::from("referrer2")
        }
    );

    check(res, expect!["empty"]);

    assert_eq!(api.referral_code_owner, Some("referrer2".to_owned()));

    assert_eq!(api.code_dapp_earnings, 0);

    let res = exec_msg_ok!(
        api,
        "dapp",
        MsgKind::Referral {
            code: ReferralCode::from(1),
        }
    );

    check(res, expect!["empty"]);

    assert_eq!(api.code_dapp_earnings, 750);

    let res = exec_msg_ok!(
        api,
        "referrer2",
        Collection::Referrer {
            code: ReferralCode::from(1),
            dapp: Id::from("dapp")
        }
    );

    check(
        res,
        expect![[r#"
            [
                WithdrawPending(Id("rewards_pot")),
                RedistributeRewards {
                    amount: 750,
                    pot: Id("rewards_pot"),
                    receiver: Id("referrer2"),
                },
            ]"#]],
    );

    api.set_dapp_total_rewards(1333);

    check(
        crate::pretty(&api),
        expect![[r#"
            MockApi {
                dapp: Some(("dapp", "dapp")),
                percent: Some(75),
                collector: Some("collector"),
                rewards_pot: Some("rewards_pot"),
                rewards_pot_admin: None,
                rewards_admin: Some("self"),
                current_fee: Some(1000),
                referral_code: Some(1),
                referral_code_owner: Some("referrer2"),
                latest_referral_code: Some(1),
                dapp_reffered_invocations: 1,
                code_total_earnings: 750,
                code_dapp_earnings: 750,
                dapp_contributions: 750,
                code_total_collected: 750,
                code_dapp_collected: 750,
                dapp_total_collected: 0,
                dapp_total_rewards: 1333,
            }"#]],
    );

    let res = exec_msg_ok!(
        api,
        "collector",
        Collection::Dapp {
            dapp: Id::from("dapp")
        }
    );

    check(
        res,
        expect![[r#"
            [
                WithdrawPending(Id("rewards_pot")),
                RedistributeRewards {
                    amount: 583,
                    pot: Id("rewards_pot"),
                    receiver: Id("collector"),
                },
            ]"#]],
    );

    let res = exec_msg_ok!(
        api,
        "collector",
        Registration::DeregisterDapp {
            dapp: Id::from("dapp"),
            rewards_admin: Id::from("collector"),
            rewards_recipient: Id::from("collector")
        }
    );

    check(
        res,
        expect![[r#"
            [
                WithdrawPending(Id("rewards_pot")),
                SetRewardsRecipient(Id("collector")),
                SetRewardsAdmin(Id("collector")),
            ]"#]],
    );
}

impl std::fmt::Display for DisplayReply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Reply::Empty => write!(f, "empty"),
            Reply::ReferralCode(code) => write!(f, "{{ code: {} }}", code.to_u64()),
            Reply::Cmd(cmd) => write!(f, "{cmd:?}"),
            Reply::MultiCmd(cmds) => {
                write!(f, "{}", pretty(cmds))
            }
        }
    }
}
