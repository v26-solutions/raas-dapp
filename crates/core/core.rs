#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use std::num::NonZeroU128;

#[derive(dbg_pls::DebugPls, Debug, thiserror::Error)]
pub enum Error<Api> {
    #[error(transparent)]
    Api(#[from] Api),
    #[error("unauthorised")]
    Unauthorized,
    #[error("already registered")]
    AlreadyRegistered,
    #[error("dapp not registered")]
    DappNotRegistered,
    #[error("referral code not registered")]
    ReferralCodeNotRegistered,
    #[error("invalid rewards admin")]
    InvalidRewardsAdmin,
    #[error("invalid rewards pot admin")]
    InvalidRewardsPotAdmin,
    #[error("rewards pot already set")]
    RewardsPotAlreadySet,
    #[error("math overflow")]
    Overflow,
    #[error("nothing to collect")]
    NothingToCollect,
}

pub mod collect;
pub mod common;
pub mod dapp;
pub mod referral;

pub use common::*;
pub use dapp::Metadata as DappMetadata;
pub use referral::Code as ReferralCode;

pub use collect::MutableStore as MutableCollectStore;
pub use collect::ReadonlyStore as ReadonlyCollectStore;
pub use dapp::MutableStore as MutableDappStore;
pub use dapp::ReadonlyStore as ReadonlyDappStore;
pub use referral::MutableStore as MutableReferralStore;
pub use referral::ReadonlyStore as ReadonlyReferralStore;

pub use collect::Query as CollectQuery;
pub use dapp::Query as DappQuery;

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
pub enum MsgKind {
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
    pub kind: MsgKind,
}

#[derive(dbg_pls::DebugPls, Debug, Clone, PartialEq)]
pub enum Command {
    /// Create a rewards pot for the given dApp Id
    CreateRewardsPot(Id),
    /// Set the given Id as the rewards recipient
    SetRewardsRecipient(Id),
    /// Set the given Id as the rewards admin
    SetRewardsAdmin(Id),
    /// Set the fee for the given dApp Id
    SetDappFee { dapp: Id, amount: NonZeroU128 },
    /// Redistribute `amount` of rewards from `pot` to `receiver`
    RedistributeRewards {
        amount: NonZeroU128,
        pot: Id,
        receiver: Id,
    },
    /// Withdraw pending rewards for Id
    WithdrawPending(Id),
}

#[derive(dbg_pls::DebugPls, Debug, Clone, PartialEq)]
pub enum Reply {
    /// Nothing to do
    Empty,
    /// Referral code to return to sender
    ReferralCode(ReferralCode),
    /// Single command to enact
    Cmd(Command),
    /// Multiple commands to enact in the given order
    MultiCmd(Vec<Command>),
}

/// Handle a message, this is the defacto entry point.
///
/// # Errors
///
/// This function will return an error if delegation of the message kind encounters an error.
pub fn exec<Api>(api: &mut Api, msg: Msg) -> Result<Reply, Error<Api::Error>>
where
    Api: ReadonlyDappStore
        + MutableDappStore
        + DappQuery
        + ReadonlyReferralStore
        + MutableReferralStore
        + ReadonlyCollectStore
        + MutableCollectStore
        + CollectQuery,
{
    match msg.kind {
        MsgKind::Register(reg) => match reg {
            Registration::Referrer => referral::register(api, msg.sender).map(Reply::from),
            Registration::Dapp {
                name,
                percent,
                collector,
            } => dapp::register(api, msg.sender, name, percent, collector).map(Reply::from),
            Registration::RewardsPot { dapp, rewards_pot } => {
                dapp::set_rewards_pot(api, &dapp, rewards_pot).map(Reply::from)
            }
            Registration::DeregisterDapp {
                dapp,
                rewards_admin,
                rewards_recipient,
            } => dapp::deregister(api, &msg.sender, &dapp, rewards_admin, rewards_recipient)
                .map(Reply::from),
        },

        MsgKind::Referral { code } => {
            referral::record(api, &msg.sender, code).map(|_| Reply::Empty)
        }

        MsgKind::Collect(collection) => match collection {
            Collection::Referrer { dapp, code } => {
                collect::referrer(api, msg.sender, &dapp, code).map(Reply::from)
            }
            Collection::Dapp { dapp } => collect::dapp(api, msg.sender, &dapp).map(Reply::from),
        },

        MsgKind::Config(configure) => match configure {
            Configure::TransferReferralCodeOwnership { code, owner } => {
                referral::transfer_ownership(api, &msg.sender, code, owner).map(|_| Reply::Empty)
            }
            Configure::DappMetadata { dapp, metadata } => {
                dapp::configure(api, &msg.sender, &dapp, metadata).map(|_| Reply::Empty)
            }
            Configure::DappFee { dapp, fee } => {
                dapp::set_fee(api, &msg.sender, dapp, fee).map(Reply::from)
            }
        },
    }
}

impl From<ReferralCode> for Reply {
    fn from(v: ReferralCode) -> Self {
        Reply::ReferralCode(v)
    }
}

impl From<Command> for Reply {
    fn from(v: Command) -> Self {
        Reply::Cmd(v)
    }
}

impl<T> From<T> for Reply
where
    T: std::iter::IntoIterator<Item = Command>,
    T::IntoIter: 'static,
{
    fn from(v: T) -> Self {
        Reply::MultiCmd(v.into_iter().collect())
    }
}

impl From<Configure> for MsgKind {
    fn from(v: Configure) -> Self {
        Self::Config(v)
    }
}

impl From<Collection> for MsgKind {
    fn from(v: Collection) -> Self {
        Self::Collect(v)
    }
}

impl From<Registration> for MsgKind {
    fn from(v: Registration) -> Self {
        Self::Register(v)
    }
}
