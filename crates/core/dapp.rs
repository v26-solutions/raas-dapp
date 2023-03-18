use std::num::NonZeroU128;

use crate::{Command, Error, Id, NonZeroPercent};

pub struct Metadata {
    pub percent: Option<NonZeroPercent>,
    pub collector: Option<Id>,
    pub repo_url: Option<String>,
}

pub trait ReadonlyStore: crate::FallibleApi {
    /// Checks whether the given `id` exists in dApp store.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn dapp_exists(&self, id: &Id) -> Result<bool, Self::Error>;

    /// Gets the percentage of a dApp's fee to give to the referrer
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn percent(&self, id: &Id) -> Result<NonZeroPercent, Self::Error>;

    /// Gets a dApp's rewards collector Id
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn collector(&self, id: &Id) -> Result<Id, Self::Error>;

    /// Checks if the dApp with the given id has a rewards pot set
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn has_rewards_pot(&self, id: &Id) -> Result<bool, Self::Error>;

    /// Gets the Id of a dApp's rewards pot
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn rewards_pot(&self, id: &Id) -> Result<Id, Self::Error>;
}

pub trait MutableStore: crate::FallibleApi {
    ///  Remove an existing dapp, once executed `dapp_exists` will return false.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn remove_dapp(&mut self, id: &Id) -> Result<(), Self::Error>;

    /// Sets the percentage of a dApp's fee to give to the referrer
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_percent(&mut self, id: &Id, percent: NonZeroPercent) -> Result<(), Self::Error>;

    /// Sets a dApp's rewards collector Id
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_collector(&mut self, id: &Id, collector: Id) -> Result<(), Self::Error>;

    /// Sets a dApp's repository url
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_repo_url(&mut self, id: &Id, repo_url: String) -> Result<(), Self::Error>;

    /// Sets the Id of a dApp's rewards pot
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_rewards_pot(&mut self, id: &Id, rewards_pot: Id) -> Result<(), Self::Error>;
}

pub trait Query: crate::FallibleApi {
    /// Returns the Id of the referral system dApp.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn self_id(&self) -> Result<Id, Self::Error>;

    /// Returns the rewards receiver `Id` of the given dApp.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn rewards_admin(&self, id: &Id) -> Result<Id, Self::Error>;

    /// Returns the admin of the given reward pot Id.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn rewards_pot_admin(&self, id: &Id) -> Result<Id, Self::Error>;

    /// Returns the current fee set by the dApp.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn current_fee(&self, id: &Id) -> Result<NonZeroU128, Self::Error>;
}

/// Registers a dApp with the system, setting at least the initial percent & collector.
///
/// # Errors
///
/// This function will return an error if:
/// - The dApp is already registered.
/// - The dApp does not have the referral program set as rewards receiver.
/// - There is an API error.
pub fn register<Api>(
    api: &mut Api,
    sender: Id,
    percent: NonZeroPercent,
    collector: Id,
) -> Result<Command, Error<Api::Error>>
where
    Api: ReadonlyStore + MutableStore + Query,
{
    if api.dapp_exists(&sender)? {
        return Err(Error::AlreadyRegistered);
    }

    if api.self_id()? != api.rewards_admin(&sender)? {
        return Err(Error::InvalidRewardsAdmin);
    }

    api.set_percent(&sender, percent)?;

    api.set_collector(&sender, collector)?;

    Ok(Command::CreateRewardsPot(sender))
}

/// Sets the rewards pot for a registered dapp
///
/// # Errors
///
/// This function will return an error if:
/// - The dApp is not registered.
/// - There is already a rewards pot set for the dApp
/// - Self ID is not the admin of the rewards pot
/// - There is an API error.
pub fn set_rewards_pot<Api>(
    api: &mut Api,
    dapp: &Id,
    rewards_pot: Id,
) -> Result<Command, Error<Api::Error>>
where
    Api: ReadonlyStore + MutableStore + Query,
{
    if !api.dapp_exists(dapp)? {
        return Err(Error::DappNotRegistered);
    }

    if api.has_rewards_pot(dapp)? {
        return Err(Error::RewardsPotAlreadySet);
    }

    if api.self_id()? != api.rewards_pot_admin(&rewards_pot)? {
        return Err(Error::InvalidRewardsPotAdmin);
    }

    api.set_rewards_pot(dapp, rewards_pot.clone())?;

    Ok(Command::SetRewardsRecipient(rewards_pot))
}

/// Deregisters a dApp in the system, collecting any outstanding rewards before relinquishing reward admin rights.
///
/// # Errors
///
/// This function will return an error if:
/// - The dApp is not registered.
/// - The sender is not either the dApp or it's collector.
/// - There is an API error.
pub fn deregister<Api>(
    api: &mut Api,
    sender: &Id,
    dapp: &Id,
    rewards_admin: Id,
    rewards_recipient: Id,
) -> Result<[Command; 3], Error<Api::Error>>
where
    Api: ReadonlyStore + MutableStore + Query,
{
    if !api.dapp_exists(dapp)? {
        return Err(Error::DappNotRegistered);
    }

    if sender != dapp && sender != &api.collector(dapp)? {
        return Err(Error::Unauthorized);
    }

    api.remove_dapp(dapp)?;

    let pot = api.rewards_pot(dapp)?;

    Ok([
        Command::WithdrawPending(pot),
        Command::SetRewardsRecipient(rewards_recipient),
        Command::SetRewardsAdmin(rewards_admin),
    ])
}

/// Configure a dApp's metadata, an action available to the dApp and it's collector.
///
/// # Errors
///
/// This function will return an error if:
/// - The dApp is not registered.
/// - The sender is not either the dApp or it's collector.
/// - There is an API error.
pub fn configure<Api>(
    api: &mut Api,
    sender: &Id,
    dapp: &Id,
    metadata: Metadata,
) -> Result<(), Error<Api::Error>>
where
    Api: ReadonlyStore + MutableStore,
{
    if !api.dapp_exists(dapp)? {
        return Err(Error::DappNotRegistered);
    }

    if sender != dapp && sender != &api.collector(dapp)? {
        return Err(Error::Unauthorized);
    }

    if let Some(percent) = metadata.percent {
        api.set_percent(dapp, percent)?;
    }

    if let Some(collector) = metadata.collector {
        api.set_collector(dapp, collector)?;
    }

    if let Some(repo) = metadata.repo_url {
        api.set_repo_url(dapp, repo)?;
    }

    Ok(())
}

/// Set a dApp's fee portion of rewards.
///
/// # Errors
///
/// This function will return an error if:
/// - The dApp is not registered.
/// - The sender is not either the dApp or it's collector.
/// - There is an API error.
pub fn set_fee<Api>(
    api: &mut Api,
    sender: &Id,
    dapp: Id,
    amount: NonZeroU128,
) -> Result<Command, Error<Api::Error>>
where
    Api: ReadonlyStore,
{
    if !api.dapp_exists(&dapp)? {
        return Err(Error::DappNotRegistered);
    }

    if sender != &dapp && sender != &api.collector(&dapp)? {
        return Err(Error::Unauthorized);
    }

    Ok(Command::SetDappFee { dapp, amount })
}
