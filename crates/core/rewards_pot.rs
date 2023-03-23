use std::num::NonZeroU128;

use crate::{FallibleApi, Id};

#[derive(dbg_pls::DebugPls, Debug, thiserror::Error)]
pub enum Error<Api> {
    #[error(transparent)]
    Api(#[from] Api),
    #[error("unauthorized")]
    Unauthorized,
}

#[derive(dbg_pls::DebugPls, Debug)]
pub enum Kind {
    WithdrawPending,
    Distribute { recipient: Id, amount: NonZeroU128 },
}

#[derive(dbg_pls::DebugPls, Debug)]
pub struct Msg {
    pub sender: Id,
    pub kind: Kind,
}

#[derive(dbg_pls::DebugPls, Debug, Clone, PartialEq)]
pub enum Command {
    WithdrawPending,
    Send { recipient: Id, amount: NonZeroU128 },
}

#[derive(dbg_pls::DebugPls, Debug, Clone, PartialEq)]
pub enum Reply {
    Empty,
    Commands(Vec<Command>),
}

pub trait Query: FallibleApi {
    /// Gets the pot owner id
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn owner_id(&self) -> Result<Id, Self::Error>;

    /// Checks if the rewards pot has any uncollected rewards
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn has_uncollected_rewards(&self) -> Result<bool, Self::Error>;
}

/// Attempt to withdraw any pending rewards
///
/// # Errors
///
/// This function will return an error if:
/// - The sender is not the owner
pub fn withdraw_pending<Api>(api: &mut Api, sender: &Id) -> Result<Vec<Command>, Error<Api::Error>>
where
    Api: Query,
{
    if sender != &api.owner_id()? {
        return Err(Error::Unauthorized);
    }

    let mut commands = vec![];

    if api.has_uncollected_rewards()? {
        commands.push(Command::WithdrawPending);
    }

    Ok(commands)
}

/// Attempt to distibute an amount of rewards to the specified recipient
///
/// # Errors
///
/// This function will return an error if:
/// - The sender is not the owner
pub fn distribute<Api>(
    api: &mut Api,
    sender: &Id,
    recipient: Id,
    amount: NonZeroU128,
) -> Result<Vec<Command>, Error<Api::Error>>
where
    Api: Query,
{
    if sender != &api.owner_id()? {
        return Err(Error::Unauthorized);
    }

    let mut commands = vec![];

    if api.has_uncollected_rewards()? {
        commands.push(Command::WithdrawPending);
    }

    // we assume that this will fail if for some reason the pot
    // has an insufficient balance
    commands.push(Command::Send { recipient, amount });

    Ok(commands)
}

pub trait HandleReply: FallibleApi {
    type Response;

    /// Consume oneself and return the final response
    fn into_response(self) -> Self::Response;

    /// Withdraw any pending rewards
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn withdraw_pending(&mut self) -> Result<(), Self::Error>;

    /// Send a given amount of rewards to a recipient
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn send_rewards(&mut self, receiver: Id, amount: NonZeroU128) -> Result<(), Self::Error>;
}

/// Handle a message, this is the defacto entry point.
///
/// # Errors
///
/// This function will return an error if delegation of the message kind encounters an error.
pub fn exec<Api>(api: &mut Api, msg: Msg) -> Result<Reply, Error<Api::Error>>
where
    Api: Query,
{
    match msg.kind {
        Kind::WithdrawPending => withdraw_pending(api, &msg.sender).map(Reply::Commands),
        Kind::Distribute { recipient, amount } => {
            distribute(api, &msg.sender, recipient, amount).map(Reply::Commands)
        }
    }
}

/// Handle a reply to core message execution
///
/// # Errors
///
/// This function will return an error if the provided API encounters an error.
pub fn handle_reply<Api>(mut api: Api, reply: Reply) -> Result<Api::Response, Api::Error>
where
    Api: HandleReply,
{
    match reply {
        Reply::Empty => {}
        Reply::Commands(cmds) => {
            for cmd in cmds {
                match cmd {
                    Command::WithdrawPending => api.withdraw_pending()?,
                    Command::Send { recipient, amount } => api.send_rewards(recipient, amount)?,
                }
            }
        }
    }

    Ok(api.into_response())
}
