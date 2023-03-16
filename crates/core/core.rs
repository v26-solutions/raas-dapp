#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use std::error::Error as StdError;
use std::num::NonZeroU128;

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

pub trait FallibleApi {
    type Error: StdError;
}

pub mod common {
    use std::num::NonZeroU128;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Id(String);

    impl Id {
        #[must_use]
        pub fn into_string(self) -> String {
            self.0
        }
    }

    impl<T> From<T> for Id
    where
        T: Into<String>,
    {
        fn from(value: T) -> Self {
            Id(value.into())
        }
    }

    impl AsRef<str> for Id {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    impl AsRef<String> for Id {
        fn as_ref(&self) -> &String {
            &self.0
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct NonZeroPercent(u8);

    impl NonZeroPercent {
        #[must_use]
        pub const fn new(percent: u8) -> Option<Self> {
            if percent == 0 || percent > 100 {
                return None;
            }

            Some(NonZeroPercent(percent))
        }

        #[must_use]
        pub const fn to_u8(self) -> u8 {
            self.0
        }

        /// Apply the percentage to a give amount, will return `None` if an overflow occurs
        #[must_use]
        pub fn checked_apply_to(self, amount: NonZeroU128) -> Option<Option<NonZeroU128>> {
            amount
                .checked_mul(self.into())
                .map(|numer| NonZeroU128::new(numer.get() / 100))
        }
    }

    impl From<NonZeroPercent> for NonZeroU128 {
        fn from(value: NonZeroPercent) -> Self {
            // safe due to checks on NonZeroPercent creation
            unsafe { NonZeroU128::new_unchecked(u128::from(value.0)) }
        }
    }
}

pub mod dapp {
    use std::num::NonZeroU128;

    use crate::{Command, Error, Id, NonZeroPercent};

    pub struct Metadata {
        pub percent: Option<NonZeroPercent>,
        pub collector: Option<Id>,
        pub repo_url: Option<String>,
    }

    pub trait Store: crate::FallibleApi {
        /// Checks whether the given `id` exists in dApp store.
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn dapp_exists(&self, id: &Id) -> Result<bool, Self::Error>;

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

        /// Gets the percentage of a dApp's fee to give to the referrer
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn percent(&self, id: &Id) -> Result<NonZeroPercent, Self::Error>;

        /// Sets a dApp's rewards collector Id
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn set_collector(&mut self, id: &Id, collector: Id) -> Result<(), Self::Error>;

        /// Gets a dApp's rewards collector Id
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn collector(&self, id: &Id) -> Result<Id, Self::Error>;

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

        /// Checks if the dApp with the given id has a rewards pot set
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn has_rewards_pot(&mut self, id: &Id) -> Result<bool, Self::Error>;

        /// Gets the Id of a dApp's rewards pot
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn rewards_pot(&self, id: &Id) -> Result<Id, Self::Error>;
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
    pub fn register<Api: Store + Query>(
        api: &mut Api,
        sender: Id,
        percent: NonZeroPercent,
        collector: Id,
    ) -> Result<Command, Error<Api::Error>> {
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
    pub fn set_rewards_pot<Api: Store + Query>(
        api: &mut Api,
        dapp: &Id,
        rewards_pot: Id,
    ) -> Result<Command, Error<Api::Error>> {
        if !api.dapp_exists(dapp)? {
            return Err(Error::DappNotRegistered);
        }

        if !api.has_rewards_pot(dapp)? {
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
    pub fn deregister<Api: Store + Query>(
        api: &mut Api,
        sender: &Id,
        dapp: &Id,
        rewards_admin: Id,
        rewards_recipient: Id,
    ) -> Result<[Command; 3], Error<Api::Error>> {
        if !api.dapp_exists(dapp)? {
            return Err(Error::DappNotRegistered);
        }

        if sender != dapp || sender != &api.collector(dapp)? {
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
    pub fn configure<Api: Store>(
        api: &mut Api,
        sender: &Id,
        dapp: &Id,
        metadata: Metadata,
    ) -> Result<(), Error<Api::Error>> {
        if !api.dapp_exists(dapp)? {
            return Err(Error::DappNotRegistered);
        }

        if sender != dapp || sender != &api.collector(dapp)? {
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
    pub fn set_fee<Api: Store>(
        api: &mut Api,
        sender: &Id,
        dapp: Id,
        amount: NonZeroU128,
    ) -> Result<Command, Error<Api::Error>> {
        if !api.dapp_exists(&dapp)? {
            return Err(Error::DappNotRegistered);
        }

        if sender != &dapp || sender != &api.collector(&dapp)? {
            return Err(Error::Unauthorized);
        }

        Ok(Command::SetDappFee { dapp, amount })
    }
}

pub mod referral {
    use std::num::NonZeroU128;

    use crate::{dapp::Query as DappQuery, dapp::Store as DappStore, Error, Id};

    #[derive(Debug, Clone, Copy)]
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

    pub trait Store: crate::FallibleApi {
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

        /// Sets the latest registered referral code.
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn set_latest(&mut self, code: Code) -> Result<(), Self::Error>;

        /// Gets the latest registered referral code.
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn latest(&self) -> Result<Code, Self::Error>;

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
        fn set_total_earnings(&mut self, code: Code, total: NonZeroU128)
            -> Result<(), Self::Error>;

        /// Gets the total earnings of a referral code.
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn total_earnings(&self, code: Code) -> Result<Option<NonZeroU128>, Self::Error>;

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

        /// Gets the earnings of a referral code per dApp.
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn dapp_earnings(&self, dapp: &Id, code: Code) -> Result<Option<NonZeroU128>, Self::Error>;

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

        /// Gets the total contributions from a dApp to all referrers.
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn dapp_contributions(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error>;
    }

    /// Register for a referral code.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The sender already has a referral code.
    /// - There is an API error.
    pub fn register<Api: Store>(api: &mut Api, sender: Id) -> Result<Code, Error<Api::Error>> {
        if api.owner_exists(&sender)? {
            return Err(Error::AlreadyRegistered);
        }

        let code = api.latest()?.next();

        api.set_code_owner(code, sender)?;

        api.set_latest(code)?;

        Ok(code)
    }

    /// Transfer ownership of a referral code
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The sender is not the current owner of the given code.
    /// - There is an API error.
    pub fn transfer_ownership<Api: Store>(
        api: &mut Api,
        sender: &Id,
        code: Code,
        new_owner: Id,
    ) -> Result<(), Error<Api::Error>> {
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
    /// - There is an API error.
    pub fn record<Api: Store + DappQuery + DappStore>(
        api: &mut Api,
        sender: &Id,
        code: Code,
    ) -> Result<(), Error<Api::Error>> {
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
}

pub mod collect {
    use std::num::NonZeroU128;

    use crate::{referral::Store as ReferralStore, Command, DappStore, Error, Id, ReferralCode};

    pub trait Store: crate::FallibleApi {
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

        /// Gets the total earnings of a referral code.
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn referrer_total_collected(
            &self,
            code: ReferralCode,
        ) -> Result<Option<NonZeroU128>, Self::Error>;

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

        /// Gets the total earnings collected on behalf of a dapp.
        ///
        /// # Errors
        ///
        /// This function will return an error depending on the implementor.
        fn dapp_total_collected(&self, dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error>;
    }

    pub trait Query: crate::FallibleApi {
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
    pub fn referrer<Api: Store + Query + ReferralStore + DappStore>(
        api: &mut Api,
        sender: Id,
        dapp: &Id,
        code: ReferralCode,
    ) -> Result<[Command; 2], Error<Api::Error>> {
        let Some(referrer_owner) = api.owner_of(code)? else {
            return Err(Error::ReferralCodeNotRegistered);
        };

        if sender != referrer_owner {
            return Err(Error::Unauthorized);
        }

        let Some(dapp_earnings) = api.dapp_earnings(dapp, code)? else {
            return Err(Error::NothingToCollect);
        };

        let Some(owed) = api
            .referrer_dapp_collected(dapp, code)?
            .and_then(|collected| NonZeroU128::new(dapp_earnings.get() - collected.get()))
            .or(Some(dapp_earnings))
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

        Ok([
            Command::WithdrawPending(pot.clone()),
            Command::RedistributeRewards {
                amount: owed,
                pot,
                receiver: sender,
            },
        ])
    }

    /// Collect a dApp's remaining rewards.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The sender is not either the dApp or it's nominated collector.
    /// - There are no rewards to collect.
    /// - There is an API error.
    pub fn dapp<Api: Store + Query + ReferralStore + DappStore>(
        api: &mut Api,
        sender: Id,
        dapp: &Id,
    ) -> Result<[Command; 2], Error<Api::Error>> {
        if &sender != dapp || sender != api.collector(dapp)? {
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

        let Some(owed) = api
            .dapp_total_collected(dapp)?
            .and_then(|collected| NonZeroU128::new(total_remaining.get() - collected.get()))
            .or(Some(total_remaining))
        else {
            return Err(Error::NothingToCollect);
        };

        api.set_dapp_total_collected(dapp, total_remaining)?;

        let pot = api.rewards_pot(dapp)?;

        Ok([
            Command::WithdrawPending(pot.clone()),
            Command::RedistributeRewards {
                amount: owed,
                pot,
                receiver: sender,
            },
        ])
    }
}

pub use common::*;
pub use dapp::Metadata as DappMetadata;
pub use referral::Code as ReferralCode;

pub use collect::Store as CollectStore;
pub use dapp::Store as DappStore;
pub use referral::Store as ReferralStore;

pub use collect::Query as CollectQuery;
pub use dapp::Query as DappQuery;

pub enum Registration {
    /// Register for a referral code
    Referrer,
    /// Dapp self-registration to take referrals
    Dapp {
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

pub enum Collection {
    /// Collect referrer earnings
    Referrer { dapp: Id, code: ReferralCode },
    /// Collect dApp remaining rewards
    Dapp { dapp: Id },
}

pub enum Configure {
    TransferReferralCodeOwnership { code: ReferralCode, owner: Id },
    DappMetadata { dapp: Id, metadata: DappMetadata },
    DappFee { dapp: Id, fee: NonZeroU128 },
}

pub enum MsgKind {
    Register(Registration),
    /// Record a referral code invocation
    Referral {
        code: ReferralCode,
    },
    Collect(Collection),
    Config(Configure),
}

pub struct Msg {
    pub sender: Id,
    pub kind: MsgKind,
}

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

pub enum Reply {
    /// Nothing to do
    Empty,
    /// Referral code to return to sender
    ReferralCode(ReferralCode),
    /// Single command to enact
    Cmd(Command),
    /// Multiple commands to enact in the given order
    MultiCmd(Box<dyn Iterator<Item = Command>>),
}

/// Handle a message, this is the defacto entry point.
///
/// # Errors
///
/// This function will return an error if delegation of the message kind encounters an error.
pub fn exec<Api>(api: &mut Api, msg: Msg) -> Result<Reply, Error<Api::Error>>
where
    Api: dapp::Store + dapp::Query + referral::Store + collect::Store + collect::Query,
{
    match msg.kind {
        MsgKind::Register(reg) => match reg {
            Registration::Referrer => referral::register(api, msg.sender).map(Reply::from),
            Registration::Dapp { percent, collector } => {
                dapp::register(api, msg.sender, percent, collector).map(Reply::from)
            }
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
        Reply::MultiCmd(Box::new(v.into_iter()))
    }
}
