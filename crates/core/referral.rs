use std::num::NonZeroU128;

use crate::{DappQuery, Error, Id, ReadonlyDappStore};

#[derive(Debug, Default, Clone, Copy)]
pub struct Code(u64);

impl Code {
    fn next(self) -> Code {
        Code(self.0 + 1)
    }

    #[must_use]
    pub fn to_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for Code {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

pub trait ReadonlyStore: crate::FallibleApi {
    /// Checks whether the given `code` exists.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn code_exists(&self, code: Code) -> Result<bool, Self::Error>;

    /// Checks whether the given `id` owns a referral code.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn owner_exists(&self, owner: &Id) -> Result<bool, Self::Error>;

    /// Gets the owner of the given code (if one exists).
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn owner_of(&self, code: Code) -> Result<Option<Id>, Self::Error>;

    /// Gets the latest registered referral code.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn latest(&self) -> Result<Option<Code>, Self::Error>;

    /// Gets the total earnings of a referral code.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn total_earnings(&self, code: Code) -> Result<Option<NonZeroU128>, Self::Error>;

    /// Gets the earnings of a referral code per dApp.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn dapp_earnings(&self, dapp: &Id, code: Code) -> Result<Option<NonZeroU128>, Self::Error>;

    /// Gets the total contributions from a dApp to all referrers.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn dapp_contributions(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error>;
}

pub trait MutableStore: crate::FallibleApi {
    /// Sets the latest registered referral code.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_latest(&mut self, code: Code) -> Result<(), Self::Error>;

    /// Sets a referral code's owner, overwriting the previous owner if any.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_code_owner(&mut self, code: Code, owner: Id) -> Result<(), Self::Error>;

    /// Increments number of invocations of a dApp by a referrer.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn increment_invocations(&mut self, dapp: &Id, code: Code) -> Result<(), Self::Error>;

    /// Sets the total earnings of a referral code.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_total_earnings(&mut self, code: Code, total: NonZeroU128) -> Result<(), Self::Error>;

    /// Sets the earnings of a referral code per dApp.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_dapp_earnings(
        &mut self,
        dapp: &Id,
        code: Code,
        total: NonZeroU128,
    ) -> Result<(), Self::Error>;

    /// Sets the total contributions from a dApp to all referrers.
    ///
    /// # Errors
    ///
    /// This function will return an error depending on the implementor.
    fn set_dapp_contributions(
        &mut self,
        dapp: &Id,
        contributions: NonZeroU128,
    ) -> Result<(), Self::Error>;
}

/// Register for a referral code.
///
/// # Errors
///
/// This function will return an error if:
/// - The sender already has a referral code.
/// - There is an API error.
pub fn register<Api>(api: &mut Api, sender: Id) -> Result<Code, Error<Api::Error>>
where
    Api: ReadonlyStore + MutableStore,
{
    if api.owner_exists(&sender)? {
        return Err(Error::AlreadyRegistered);
    }

    let code = api.latest()?.unwrap_or_default().next();

    api.set_code_owner(code, sender)?;

    api.set_latest(code)?;

    Ok(code)
}

/// Transfer ownership of a referral code
///
/// # Errors
///
/// This function will return an error if:
/// - The referral code is not registered.
/// - The sender is not the current owner of the given code.
/// - There is an API error.
pub fn transfer_ownership<Api>(
    api: &mut Api,
    sender: &Id,
    code: Code,
    new_owner: Id,
) -> Result<(), Error<Api::Error>>
where
    Api: ReadonlyStore + MutableStore,
{
    let Some(current_owner) = api.owner_of(code)? else {
        return Err(Error::ReferralCodeNotRegistered);
    };

    if sender != &current_owner {
        return Err(Error::Unauthorized);
    }

    api.set_code_owner(code, new_owner)?;

    Ok(())
}

/// Record an invocation with a referral code.
///
/// # Errors
///
/// This function will return an error if:
/// - The sender is not a registered dApp.
/// - The referral code does not exist.
/// - Calculated earnings/contributions overflow 128-bits.
/// - There is an API error.
pub fn record<Api>(api: &mut Api, sender: &Id, code: Code) -> Result<(), Error<Api::Error>>
where
    Api: ReadonlyStore + MutableStore + DappQuery + ReadonlyDappStore,
{
    if !api.dapp_exists(sender)? {
        return Err(Error::DappNotRegistered);
    }

    if !api.code_exists(code)? {
        return Err(Error::ReferralCodeNotRegistered);
    }

    api.increment_invocations(sender, code)?;

    let current_fee = api.current_fee(sender)?;

    let Some(referrer_share) = api
        .percent(sender)?
        .checked_apply_to(current_fee)
        .ok_or(Error::Overflow)?
    else {
        return Ok(());
    };

    let total_earnings = match api.total_earnings(code)? {
        Some(cur) => cur
            .checked_add(referrer_share.get())
            .ok_or(Error::Overflow)?,
        None => referrer_share,
    };

    api.set_total_earnings(code, total_earnings)?;

    let dapp_earnings = match api.dapp_earnings(sender, code)? {
        Some(cur) => cur
            .checked_add(referrer_share.get())
            .ok_or(Error::Overflow)?,
        None => referrer_share,
    };

    api.set_dapp_earnings(sender, code, dapp_earnings)?;

    let dapp_contributions = match api.dapp_contributions(sender)? {
        Some(cur) => cur
            .checked_add(referrer_share.get())
            .ok_or(Error::Overflow)?,
        None => referrer_share,
    };

    api.set_dapp_contributions(sender, dapp_contributions)?;

    Ok(())
}
