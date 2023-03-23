#[derive(Debug, thiserror::Error)]
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
pub mod msg;
pub mod referral;
pub mod reply;

pub use common::*;
pub use dapp::Metadata as DappMetadata;
pub use msg::*;
pub use referral::Code as ReferralCode;

pub use collect::MutableStore as MutableCollectStore;
pub use collect::ReadonlyStore as ReadonlyCollectStore;

pub use dapp::MutableStore as MutableDappStore;
pub use dapp::ReadonlyStore as ReadonlyDappStore;

pub use referral::MutableStore as MutableReferralStore;
pub use referral::ReadonlyStore as ReadonlyReferralStore;

pub use collect::Query as CollectQuery;
pub use dapp::Query as DappQuery;

pub use reply::handle as handle_reply;
pub use reply::Handle as HandleReply;
pub use reply::{Command, Reply};

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
        Kind::Register(reg) => match reg {
            Registration::Referrer => referral::register(api, msg.sender).map(Reply::from),
            Registration::Dapp {
                name,
                percent,
                collector,
            } => dapp::register(api, msg.sender, name, percent, collector).map(Reply::from),
            Registration::RewardsPot { dapp, rewards_pot } => {
                dapp::set_rewards_pot(api, dapp, rewards_pot).map(Reply::from)
            }
            Registration::DeregisterDapp {
                dapp,
                rewards_admin,
                rewards_recipient,
            } => dapp::deregister(api, &msg.sender, dapp, rewards_admin, rewards_recipient)
                .map(Reply::from),
        },

        Kind::Referral { code } => referral::record(api, &msg.sender, code).map(|_| Reply::Empty),

        Kind::Collect(collection) => match collection {
            Collection::Referrer { dapp, code } => {
                collect::referrer(api, msg.sender, &dapp, code).map(Reply::from)
            }
            Collection::Dapp { dapp } => collect::dapp(api, msg.sender, &dapp).map(Reply::from),
        },

        Kind::Config(configure) => match configure {
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
