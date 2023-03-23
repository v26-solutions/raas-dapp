use std::num::NonZeroU128;

use crate::{FallibleApi, Id};

use super::ReferralCode;

pub trait Handle: FallibleApi {
    type Response;

    /// Consume oneself and return the final response
    fn into_response(self) -> Self::Response;

    /// Add the referral code to the response
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn add_referral_code(&mut self, referral_code: ReferralCode) -> Result<(), Self::Error>;

    /// Create the rewards pot
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn create_rewards_pot(&mut self, dapp: Id) -> Result<(), Self::Error>;

    /// Set the rewards recipient
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_rewards_recipient(&mut self, dapp: Id, recipient: Id) -> Result<(), Self::Error>;

    /// Set the rewards admin
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_rewards_admin(&mut self, dapp: Id, admin: Id) -> Result<(), Self::Error>;

    /// Set a dApp's fee
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_dapp_fee(&mut self, dapp: Id, amount: NonZeroU128) -> Result<(), Self::Error>;

    /// Withdraw any pending rewards for the given pot
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn withdraw_rewards(&mut self, pot: Id) -> Result<(), Self::Error>;

    /// Distribute an amount of rewards from the rewards pot to a receiver.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn distribute_rewards(
        &mut self,
        pot: Id,
        amount: NonZeroU128,
        receiver: Id,
    ) -> Result<(), Self::Error>;
}

#[derive(dbg_pls::DebugPls, Debug, Clone, PartialEq)]
pub enum Command {
    /// Create a rewards pot for the given dApp Id
    CreateRewardsPot(Id),
    /// Set a dApp's reward recipient
    SetRewardsRecipient { dapp: Id, recipient: Id },
    /// Set a dApp's reward admin
    SetRewardsAdmin { dapp: Id, admin: Id },
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

/// Handle a command issued after core message execution
///
/// # Errors
///
/// This function will return an error if the provided API encounters an error.
pub fn handle_cmd<Api>(api: &mut Api, cmd: Command) -> Result<(), Api::Error>
where
    Api: Handle,
{
    match cmd {
        Command::CreateRewardsPot(dapp) => api.create_rewards_pot(dapp),
        Command::SetRewardsRecipient { dapp, recipient } => {
            api.set_rewards_recipient(dapp, recipient)
        }
        Command::SetRewardsAdmin { dapp, admin } => api.set_rewards_admin(dapp, admin),
        Command::SetDappFee { dapp, amount } => api.set_dapp_fee(dapp, amount),
        Command::RedistributeRewards {
            amount,
            pot,
            receiver,
        } => api.distribute_rewards(pot, amount, receiver),
        Command::WithdrawPending(pot) => api.withdraw_rewards(pot),
    }
}

/// Handle a reply to core message execution
///
/// # Errors
///
/// This function will return an error if the provided API encounters an error.
pub fn handle<Api>(mut api: Api, reply: Reply) -> Result<Api::Response, Api::Error>
where
    Api: Handle,
{
    match reply {
        Reply::Empty => {}
        Reply::ReferralCode(code) => api.add_referral_code(code)?,
        Reply::Cmd(cmd) => handle_cmd(&mut api, cmd)?,
        Reply::MultiCmd(cmds) => {
            for cmd in cmds {
                handle_cmd(&mut api, cmd)?;
            }
        }
    }

    Ok(api.into_response())
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
