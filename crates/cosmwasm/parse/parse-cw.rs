#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use std::num::NonZeroU128;

use cosmwasm_std::{Api, MessageInfo, Reply, StdError};

use cw_utils::ParseReplyError;

use referrals_core::hub::{
    Collection, Configure, DappMetadata, Kind as HubMsgKind, Msg as HubMsg, NonZeroPercent,
    ReferralCode, Registration,
};
use referrals_core::rewards_pot::{Kind as RewardsPotKind, Msg as RewardsPotMsg};
use referrals_core::Id;

use referrals_cw::rewards_pot::ExecuteMsg as PotExecuteMsg;
use referrals_cw::rewards_pot::InstantiateResponse as PotInitResponse;
use referrals_cw::ExecuteMsg as HubExecuteMsg;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid address - {0}")]
    InvalidAddress(#[from] StdError),
    #[error("invalid percent - valid value is any integer between 1 & 100")]
    InvalidPercent,
    #[error("invalid fee - expected non-zero value")]
    InvalidFee,
    #[error("invalid amount - expected non-zero value")]
    InvalidAmount,
    #[error(transparent)]
    Reply(#[from] ParseReplyError),
    #[error("invalid reply - expected data")]
    ExpectedReplyData,
    #[error("invalid reply - error parsing data - {0}")]
    InvalidReplyData(StdError),
}

/// Parse an untrusted user provided `referrals_core::hub::ExecuteMsg` into a trusted core msg
///
/// # Errors
///
/// This function will return an error if the user provided message contains invalid fields.
pub fn parse_hub_exec(
    api: &dyn Api,
    msg_info: MessageInfo,
    cw_msg: HubExecuteMsg,
) -> Result<HubMsg, Error> {
    let kind = match cw_msg {
        HubExecuteMsg::RegisterReferrer {} => HubMsgKind::Register(Registration::Referrer),

        HubExecuteMsg::RegisterDapp {
            name,
            percent,
            collector,
        } => HubMsgKind::Register(Registration::Dapp {
            name,
            percent: NonZeroPercent::new(percent).ok_or(Error::InvalidPercent)?,
            collector: api.addr_validate(&collector).map(Id::from)?,
        }),

        HubExecuteMsg::DeregisterDapp {
            dapp,
            rewards_admin,
            rewards_recipient,
        } => HubMsgKind::Register(Registration::DeregisterDapp {
            dapp: api.addr_validate(&dapp).map(Id::from)?,
            rewards_admin: api.addr_validate(&rewards_admin).map(Id::from)?,
            rewards_recipient: api.addr_validate(&rewards_recipient).map(Id::from)?,
        }),

        HubExecuteMsg::SetDappFee { dapp, fee } => HubMsgKind::Config(Configure::DappFee {
            dapp: api.addr_validate(&dapp).map(Id::from)?,
            fee: NonZeroU128::new(fee.u128()).ok_or(Error::InvalidFee)?,
        }),

        HubExecuteMsg::RecordReferral { code } => HubMsgKind::Referral {
            code: ReferralCode::from(code),
        },

        HubExecuteMsg::CollectReferrer { code, dapp } => {
            HubMsgKind::Collect(Collection::Referrer {
                dapp: api.addr_validate(&dapp).map(Id::from)?,
                code: ReferralCode::from(code),
            })
        }

        HubExecuteMsg::CollectDapp { dapp } => HubMsgKind::Collect(Collection::Dapp {
            dapp: api.addr_validate(&dapp).map(Id::from)?,
        }),

        HubExecuteMsg::TransferOwnership { code, owner } => {
            HubMsgKind::Config(Configure::TransferReferralCodeOwnership {
                code: ReferralCode::from(code),
                owner: api.addr_validate(&owner).map(Id::from)?,
            })
        }

        HubExecuteMsg::ConfigureDapp {
            dapp,
            percent,
            collector,
            repo_url,
        } => HubMsgKind::Config(Configure::DappMetadata {
            dapp: api.addr_validate(&dapp).map(Id::from)?,
            metadata: DappMetadata {
                percent: percent
                    .map(|p| NonZeroPercent::new(p).ok_or(Error::InvalidPercent))
                    .transpose()?,
                collector: collector
                    .map(|c| api.addr_validate(&c).map(Id::from).map_err(Error::from))
                    .transpose()?,
                repo_url,
            },
        }),
    };

    Ok(HubMsg {
        sender: Id::from(msg_info.sender),
        kind,
    })
}

/// Parse a trusted cosmwasm reply into a core msg
///
/// # Errors
///
/// This function will return an error if the reply is invalid.
pub fn parse_init_pot_reply(reply: Reply) -> Result<HubMsg, Error> {
    let contract_init_res = cw_utils::parse_reply_instantiate_data(reply)?;

    let pot_init_response: PotInitResponse = contract_init_res
        .data
        .ok_or(Error::ExpectedReplyData)
        .and_then(|data| cosmwasm_std::from_binary(&data).map_err(Error::InvalidReplyData))?;

    let rewards_pot = Id::from(contract_init_res.contract_address);

    Ok(HubMsg {
        sender: rewards_pot.clone(),
        kind: HubMsgKind::Register(Registration::RewardsPot {
            dapp: Id::from(pot_init_response.dapp),
            rewards_pot,
        }),
    })
}

/// Parse an untrusted user provided `referrals_core::rewards_pot::ExecuteMsg` into a trusted core msg
///
/// # Errors
///
/// This function will return an error if the user provided message contains invalid fields.
pub fn parse_pot_exec(
    api: &dyn Api,
    msg_info: MessageInfo,
    cw_msg: PotExecuteMsg,
) -> Result<RewardsPotMsg, Error> {
    let kind = match cw_msg {
        PotExecuteMsg::WithdrawRewards {} => RewardsPotKind::WithdrawPending,
        PotExecuteMsg::DistributeRewards { recipient, amount } => RewardsPotKind::Distribute {
            recipient: api.addr_validate(&recipient).map(Id::from)?,
            amount: NonZeroU128::new(amount.u128()).ok_or(Error::InvalidAmount)?,
        },
    };

    Ok(RewardsPotMsg {
        sender: Id::from(msg_info.sender),
        kind,
    })
}
