use std::num::NonZeroU128;

use crate::Id;

use super::{DappMetadata, NonZeroPercent, ReferralCode};

#[derive(dbg_pls::DebugPls, Debug)]
pub enum Registration {
    /// Register for a referral code
    Referrer,
    /// Dapp self-registration to take referrals
    Dapp {
        name: String,
        percent: NonZeroPercent,
        collector: Id,
    },
    /// Set the rewards pot for the given dApp
    RewardsPot { dapp: Id, rewards_pot: Id },
    /// Dapp de-registration to stop taking referrals
    DeregisterDapp {
        dapp: Id,
        rewards_admin: Id,
        rewards_recipient: Id,
    },
}

#[derive(dbg_pls::DebugPls, Debug)]
pub enum Collection {
    /// Collect referrer earnings
    Referrer { dapp: Id, code: ReferralCode },
    /// Collect dApp remaining rewards
    Dapp { dapp: Id },
}

#[derive(dbg_pls::DebugPls, Debug)]
pub enum Configure {
    TransferReferralCodeOwnership { code: ReferralCode, owner: Id },
    DappMetadata { dapp: Id, metadata: DappMetadata },
    DappFee { dapp: Id, fee: NonZeroU128 },
}

#[derive(dbg_pls::DebugPls, Debug)]
pub enum Kind {
    Register(Registration),
    /// Record a referral code invocation
    Referral {
        code: ReferralCode,
    },
    Collect(Collection),
    Config(Configure),
}

#[derive(dbg_pls::DebugPls, Debug)]
pub struct Msg {
    pub sender: Id,
    pub kind: Kind,
}

impl From<Configure> for Kind {
    fn from(v: Configure) -> Self {
        Self::Config(v)
    }
}

impl From<Collection> for Kind {
    fn from(v: Collection) -> Self {
        Self::Collect(v)
    }
}

impl From<Registration> for Kind {
    fn from(v: Registration) -> Self {
        Self::Register(v)
    }
}
