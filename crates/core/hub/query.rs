use std::num::NonZeroU128;

use crate::{FallibleApi, Id};

use super::{
    CollectQuery, DappExternalQuery, Error, NonZeroPercent, ReadonlyDappStore,
    ReadonlyReferralStore, ReferralCode,
};

pub trait Dapps: FallibleApi {
    /// Total number of dApps that have ever been activated.
    ///
    /// # Errors
    ///
    /// This function will return an error if the implementor encounters an error.
    fn total_dapp_count(&self) -> Result<u64, Self::Error>;

    /// All the dApp ids in the order they were first activated.
    /// Supports optional pagination, by specifying `start` & `limit`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the implementor encounters an error.
    fn all_dapp_ids(&self, start: Option<u64>, limit: Option<u64>) -> Result<Vec<Id>, Self::Error>;

    /// Get the name of the dApp with given `id`, if it exists.
    ///
    /// # Errors
    ///
    /// This function will return an error if the implementor encounters an error.
    fn dapp_name(&self, dapp: &Id) -> Result<Option<String>, Self::Error>;

    /// Get the repo url of the dApp with given `id`, if one has been set
    ///
    /// # Errors
    ///
    /// This function will return an error if the implementor encounters an error.
    fn dapp_repo_url(&self, dapp: &Id) -> Result<Option<String>, Self::Error>;

    /// Get the total number of invocations from referrers for the dApp with the given `id`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the implementor encounters an error.
    fn dapp_total_invocations(&self, dapp: &Id) -> Result<u64, Self::Error>;

    /// Get the number of discrete referrers for the dApp with the given `id`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the implementor encounters an error.
    fn dapp_discrete_referrers(&self, dapp: &Id) -> Result<u64, Self::Error>;
}

pub trait Referrers: FallibleApi {
    /// `ReferralCode` registered to the `referrer`, if any.
    ///
    /// # Errors
    ///
    /// This function will return an error if the implementor encounters an error.
    fn referral_code(&self, referrer: &Id) -> Result<Option<ReferralCode>, Self::Error>;
}

pub struct DappInfo {
    pub id: Id,
    pub active: bool,
    pub name: Option<String>,
    pub percent: NonZeroPercent,
    pub repo_url: Option<String>,
    pub fee: Option<NonZeroU128>,
    pub total_invocations: u64,
    pub discrete_referrers: u64,
    pub total_contributions: u128,
    pub total_rewards: u128,
}

pub enum Request {
    TotalDappCount,
    Dapp(Id),
    AllDapps {
        start: Option<u64>,
        limit: Option<u64>,
    },
    ReferralCode(Id),
}

pub enum Response {
    TotalDappCount(u64),
    Dapp(DappInfo),
    AllDapps(Vec<DappInfo>),
    ReferralCode(Option<ReferralCode>),
}

/// All the info for the dApp with the given `id`.
///
/// # Errors
///
/// This function will return an error if:
/// - There is an API error.
pub fn dapp_info<Api>(api: &Api, id: Id) -> Result<DappInfo, Error<Api::Error>>
where
    Api: ReadonlyDappStore + Dapps + DappExternalQuery + ReadonlyReferralStore + CollectQuery,
{
    let name = api.dapp_name(&id)?;
    let percent = api.percent(&id)?;
    let repo_url = api.dapp_repo_url(&id)?;
    let fee = api.current_fee(&id)?;
    let total_invocations = api.dapp_total_invocations(&id)?;
    let discrete_referrers = api.dapp_discrete_referrers(&id)?;
    let total_contributions = api.dapp_contributions(&id)?.map_or(0, NonZeroU128::get);
    let rewards_pot = api.rewards_pot(&id)?;
    let total_rewards = api
        .dapp_total_rewards(&rewards_pot)?
        .map_or(0, NonZeroU128::get);

    let active = name.is_some() && fee.is_some();

    Ok(DappInfo {
        id,
        active,
        name,
        percent,
        repo_url,
        fee,
        total_invocations,
        discrete_referrers,
        total_contributions,
        total_rewards,
    })
}

/// All the dApps in the order they were first activated, respecting the pagination parameters if specified.
///
/// # Errors
///
/// This function will return an error if:
/// - There is an API error.
pub fn all_dapps<Api>(
    api: &Api,
    start: Option<u64>,
    limit: Option<u64>,
) -> Result<Vec<DappInfo>, Error<Api::Error>>
where
    Api: ReadonlyDappStore + Dapps + DappExternalQuery + ReadonlyReferralStore + CollectQuery,
{
    api.all_dapp_ids(start, limit)?
        .into_iter()
        .try_fold(Vec::new(), |mut dapps, id| {
            let dapp = dapp_info(api, id)?;
            dapps.push(dapp);
            Ok(dapps)
        })
}

/// Handle a query request.
///
/// # Errors
///
/// This function will return an error if delegation of the message kind encounters an error.
pub fn handle<Api>(api: &Api, request: Request) -> Result<Response, Error<Api::Error>>
where
    Api: Dapps
        + ReadonlyDappStore
        + DappExternalQuery
        + Referrers
        + ReadonlyReferralStore
        + CollectQuery,
{
    match request {
        Request::TotalDappCount => api
            .total_dapp_count()
            .map(Response::TotalDappCount)
            .map_err(Error::from),
        Request::Dapp(id) => dapp_info(api, id).map(Response::Dapp),
        Request::AllDapps { start, limit } => all_dapps(api, start, limit).map(Response::AllDapps),
        Request::ReferralCode(id) => api
            .referral_code(&id)
            .map(Response::ReferralCode)
            .map_err(Error::from),
    }
}
