#[cfg(test)]
use referrals_core::dapp;
use referrals_core::{DappQuery, DappStore};

#[cfg(test)]
use crate::{check, debug, debug_slice, expect, pretty};

use super::*;

impl DappStore for MockApi {
    fn dapp_exists(&self, id: &Id) -> Result<bool, Self::Error> {
        Ok(self.dapp.as_ref().map_or(false, |dapp| dapp == id.as_str()))
    }

    fn remove_dapp(&mut self, id: &Id) -> Result<(), Self::Error> {
        if self.dapp_exists(id)? {
            self.dapp.take();
        }

        Ok(())
    }

    fn set_percent(&mut self, id: &Id, percent: NonZeroPercent) -> Result<(), Self::Error> {
        self.dapp = Some(id.as_str().into());
        self.percent = Some(percent.to_u8());
        Ok(())
    }

    fn percent(&self, _id: &Id) -> Result<NonZeroPercent, Self::Error> {
        Ok(self.percent.and_then(NonZeroPercent::new).unwrap())
    }

    fn set_collector(&mut self, id: &Id, collector: Id) -> Result<(), Self::Error> {
        self.dapp = Some(id.as_str().into());
        self.collector = Some(collector.into_string());
        Ok(())
    }

    fn collector(&self, _id: &Id) -> Result<Id, Self::Error> {
        Ok(self.collector.as_ref().map(Id::from).unwrap())
    }

    fn set_repo_url(&mut self, id: &Id, _repo_url: String) -> Result<(), Self::Error> {
        self.dapp = Some(id.as_str().into());
        Ok(())
    }

    fn set_rewards_pot(&mut self, id: &Id, rewards_pot: Id) -> Result<(), Self::Error> {
        self.dapp = Some(id.as_str().into());
        self.rewards_pot = Some(rewards_pot.into_string());
        Ok(())
    }

    fn has_rewards_pot(&mut self, id: &Id) -> Result<bool, Self::Error> {
        Ok(self.dapp_exists(id)? && self.rewards_pot.is_some())
    }

    fn rewards_pot(&self, _id: &Id) -> Result<Id, Self::Error> {
        Ok(self.rewards_pot.as_ref().map(Id::from).unwrap())
    }
}

pub const SELF_ID: &str = "self";

impl DappQuery for MockApi {
    fn self_id(&self) -> Result<Id, Self::Error> {
        Ok(Id::from(SELF_ID))
    }

    fn rewards_admin(&self, _id: &Id) -> Result<Id, Self::Error> {
        Ok(self.rewards_admin.as_ref().map(Id::from).unwrap())
    }

    fn rewards_pot_admin(&self, _id: &Id) -> Result<Id, Self::Error> {
        Ok(self
            .rewards_pot_admin
            .as_ref()
            .map_or_else(|| Id::from(SELF_ID), Id::from))
    }

    fn current_fee(&self, _id: &Id) -> Result<NonZeroU128, Self::Error> {
        Ok(self.current_fee.unwrap())
    }
}

#[cfg(test)]
pub mod configure;
#[cfg(test)]
pub mod deregister;
#[cfg(test)]
pub mod register;
#[cfg(test)]
pub mod set_fee;
#[cfg(test)]
pub mod set_rewards_pot;
