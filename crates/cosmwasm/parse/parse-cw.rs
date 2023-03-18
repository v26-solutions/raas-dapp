#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use cosmwasm_std::{Api, MessageInfo, Reply, StdError};

use cw_utils::ParseReplyError;
use referrals_core::{
    Collection, Configure, DappMetadata, Id, Msg as CoreMsg, MsgKind as CoreMsgKind,
    NonZeroPercent, ReferralCode, Registration,
};
use referrals_cw::rewards_pot::InstantiateResponse as PotInitResponse;
use referrals_cw::ExecuteMsg as ReferralsExecuteMsg;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid address - {0}")]
    InvalidAddress(#[from] StdError),
    #[error("invalid percent - valid value is any integer between 1 & 100")]
    InvalidPercent,
    #[error(transparent)]
    Reply(#[from] ParseReplyError),
    #[error("invalid reply - expected data")]
    ExpectedReplyData,
    #[error("invalid reply - error parsing data - {0}")]
    InvalidReplyData(StdError),
}

/// Parse an untrusted user provided `ExecuteMsg` into a trusted core msg
///
/// # Errors
///
/// This function will return an error if the user provided message contains invalid fields.
pub fn parse_exec(
    api: &dyn Api,
    msg_info: MessageInfo,
    cw_msg: ReferralsExecuteMsg,
) -> Result<CoreMsg, Error> {
    let kind = match cw_msg {
        ReferralsExecuteMsg::RegisterReferrer {} => CoreMsgKind::Register(Registration::Referrer),

        ReferralsExecuteMsg::RegisterDapp {
            name,
            percent,
            collector,
        } => CoreMsgKind::Register(Registration::Dapp {
            name,
            percent: NonZeroPercent::new(percent).ok_or(Error::InvalidPercent)?,
            collector: api.addr_validate(&collector).map(Id::from)?,
        }),

        ReferralsExecuteMsg::DeregisterDapp {
            dapp,
            rewards_admin,
            rewards_recipient,
        } => CoreMsgKind::Register(Registration::DeregisterDapp {
            dapp: api.addr_validate(&dapp).map(Id::from)?,
            rewards_admin: api.addr_validate(&rewards_admin).map(Id::from)?,
            rewards_recipient: api.addr_validate(&rewards_recipient).map(Id::from)?,
        }),

        ReferralsExecuteMsg::RecordReferral { code } => CoreMsgKind::Referral {
            code: ReferralCode::from(code),
        },

        ReferralsExecuteMsg::CollectReferrer { code, dapp } => {
            CoreMsgKind::Collect(Collection::Referrer {
                dapp: api.addr_validate(&dapp).map(Id::from)?,
                code: ReferralCode::from(code),
            })
        }

        ReferralsExecuteMsg::CollectDapp { dapp } => CoreMsgKind::Collect(Collection::Dapp {
            dapp: api.addr_validate(&dapp).map(Id::from)?,
        }),

        ReferralsExecuteMsg::TransferOwnership { code, owner } => {
            CoreMsgKind::Config(Configure::TransferReferralCodeOwnership {
                code: ReferralCode::from(code),
                owner: api.addr_validate(&owner).map(Id::from)?,
            })
        }

        ReferralsExecuteMsg::ConfigureDapp {
            dapp,
            percent,
            collector,
            repo_url,
        } => CoreMsgKind::Config(Configure::DappMetadata {
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

    Ok(CoreMsg {
        sender: Id::from(msg_info.sender),
        kind,
    })
}

/// Parse a trusted cosmwasm reply into a core msg
///
/// # Errors
///
/// This function will return an error if the reply is invalid.
pub fn parse_init_pot_reply(reply: Reply) -> Result<CoreMsg, Error> {
    let contract_init_res = cw_utils::parse_reply_instantiate_data(reply)?;

    let pot_init_response: PotInitResponse = contract_init_res
        .data
        .ok_or(Error::ExpectedReplyData)
        .and_then(|data| cosmwasm_std::from_binary(&data).map_err(Error::InvalidReplyData))?;

    let rewards_pot = Id::from(contract_init_res.contract_address);

    Ok(CoreMsg {
        sender: rewards_pot.clone(),
        kind: CoreMsgKind::Register(Registration::RewardsPot {
            dapp: Id::from(pot_init_response.dapp),
            rewards_pot,
        }),
    })
}
