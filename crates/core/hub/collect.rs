use std::num::NonZeroU128;

use crate::{FallibleApi, Id};

use super::{Command, Error, ReadonlyDappStore, ReadonlyReferralStore, ReferralCode};

pub trait ReadonlyStore: FallibleApi {
    /// Gets the total earnings of a referral code.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn referrer_total_collected(
        &self,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error>;

    /// Gets the total earnings of a referral code.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn referrer_dapp_collected(
        &self,
        dapp: &Id,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error>;

    /// Gets the total earnings collected on behalf of a dapp.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn dapp_total_collected(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error>;
}

pub trait MutableStore: FallibleApi {
    /// Sets the total collected earnings for a referral code.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_referrer_total_collected(
        &mut self,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error>;

    /// Sets the collected earnings for a referral code per dApp.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_referrer_dapp_collected(
        &mut self,
        dapp: &Id,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error>;

    /// Sets the total earnings collected on behalf of a dapp.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_dapp_total_collected(
        &mut self,
        dapp: &Id,
        total: NonZeroU128,
    ) -> Result<(), Self::Error>;
}

pub trait Query: FallibleApi {
    /// The total rewards earned since dapp registration.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn dapp_total_rewards(&self, pot: &Id) -> Result<Option<NonZeroU128>, Self::Error>;
}

/// Collect a referrers earnings for a specific dApp.
///
/// # Errors
///
/// This function will return an error if:
/// - The referral code is not registered.
/// - The sender is not the owner of the referral code.
/// - There are no earnings to collect.
/// - There is an API error.
pub fn referrer<Api>(
    api: &mut Api,
    sender: Id,
    dapp: &Id,
    code: ReferralCode,
) -> Result<Command, Error<Api::Error>>
where
    Api: ReadonlyStore + MutableStore + Query + ReadonlyReferralStore + ReadonlyDappStore,
{
    let Some(referrer_owner) = api.owner_of(code)? else {
        return Err(Error::ReferralCodeNotRegistered);
    };

    if sender != referrer_owner {
        return Err(Error::Unauthorized);
    }

    let Some(dapp_earnings) = api.dapp_earnings(dapp, code)? else {
        return Err(Error::NothingToCollect);
    };

    let already_collected = api.referrer_dapp_collected(dapp, code)?;

    let Some(owed) = already_collected
        .and_then(|collected| NonZeroU128::new(dapp_earnings.get() - collected.get()))
        .or_else(|| already_collected.is_none().then_some(dapp_earnings))
    else {
        return Err(Error::NothingToCollect);
    };

    let total_collected = match api.referrer_total_collected(code)? {
        Some(total) => total.checked_add(owed.get()).ok_or(Error::Overflow)?,
        None => owed,
    };

    api.set_referrer_total_collected(code, total_collected)?;

    api.set_referrer_dapp_collected(dapp, code, dapp_earnings)?;

    let pot = api.rewards_pot(dapp)?;

    Ok(Command::RedistributeRewards {
        amount: owed,
        pot,
        receiver: sender,
    })
}

/// Collect a dApp's remaining rewards.
///
/// # Errors
///
/// This function will return an error if:
/// - The sender is not either the dApp or it's nominated collector.
/// - There are no rewards to collect.
/// - There is an API error.
pub fn dapp<Api>(api: &mut Api, sender: Id, dapp: &Id) -> Result<Command, Error<Api::Error>>
where
    Api: ReadonlyStore + MutableStore + Query + ReadonlyReferralStore + ReadonlyDappStore,
{
    if &sender != dapp && sender != api.collector(dapp)? {
        return Err(Error::Unauthorized);
    }

    let pot = api.rewards_pot(dapp)?;

    let Some(total_rewards) = api.dapp_total_rewards(&pot)? else {
        return Err(Error::NothingToCollect);
    };

    let Some(total_remaining) = api
        .dapp_contributions(dapp)?
        .and_then(|contributions| NonZeroU128::new(total_rewards.get() - contributions.get()))
    else {
        return Err(Error::NothingToCollect);
    };

    let already_collected = api.dapp_total_collected(dapp)?;

    let Some(owed) = already_collected
        .and_then(|collected| NonZeroU128::new(total_remaining.get() - collected.get()))
        .or_else(|| already_collected.is_none().then_some(total_remaining))
    else {
        return Err(Error::NothingToCollect);
    };

    api.set_dapp_total_collected(dapp, total_remaining)?;

    let pot = api.rewards_pot(dapp)?;

    Ok(Command::RedistributeRewards {
        amount: owed,
        pot,
        receiver: sender,
    })
}
