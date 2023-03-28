use std::num::NonZeroU128;

use serde::{Deserialize, Serialize};

use crate::Id;

use super::{DappMetadata, NonZeroPercent, ReferralCode};

#[derive(Serialize, Deserialize, Debug)]
pub enum Registration {
    /// Register for a referral code
    Referrer,
    /// Dapp self-activation to take referrals
    ActivateDapp {
        name: String,
        percent: NonZeroPercent,
        collector: Id,
    },
    /// Set the rewards pot for the given dApp
    RewardsPot { dapp: Id, rewards_pot: Id },
    /// Dapp de-activation to stop taking referrals
    DeactivateDapp {
        dapp: Id,
        rewards_admin: Id,
        rewards_recipient: Id,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Collection {
    /// Collect referrer earnings
    Referrer { dapp: Id, code: ReferralCode },
    /// Collect dApp remaining rewards
    Dapp { dapp: Id },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Configure {
    TransferReferralCodeOwnership { code: ReferralCode, owner: Id },
    DappMetadata { dapp: Id, metadata: DappMetadata },
    DappFee { dapp: Id, fee: NonZeroU128 },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Kind {
    Register(Registration),
    /// Record a referral code invocation
    Referral {
        code: ReferralCode,
    },
    Collect(Collection),
    Config(Configure),
}

#[derive(Serialize, Deserialize, Debug)]
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
