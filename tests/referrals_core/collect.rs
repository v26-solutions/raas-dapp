#[cfg(test)]
use referrals_core::collect;
use referrals_core::{
    CollectQuery, MutableCollectStore, ReadonlyCollectStore, ReadonlyReferralStore, ReferralCode,
};

#[cfg(test)]
use crate::{check, expect, pretty};

use super::*;

impl ReadonlyCollectStore for MockApi {
    fn referrer_total_collected(
        &self,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        assert!(self.code_exists(code)?);
        Ok(NonZeroU128::new(self.code_total_collected))
    }

    fn referrer_dapp_collected(
        &self,
        _dapp: &Id,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        assert!(self.code_exists(code)?);
        Ok(NonZeroU128::new(self.code_dapp_collected))
    }

    fn dapp_total_collected(&self, _dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        Ok(NonZeroU128::new(self.dapp_total_collected))
    }
}

impl MutableCollectStore for MockApi {
    fn set_referrer_total_collected(
        &mut self,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        assert!(self.code_exists(code)?);
        self.code_total_collected = total.get();
        Ok(())
    }

    fn set_referrer_dapp_collected(
        &mut self,
        _dapp: &Id,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        assert!(self.code_exists(code)?);
        self.code_dapp_collected = total.get();
        Ok(())
    }

    fn set_dapp_total_collected(
        &mut self,
        _dapp: &Id,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        self.dapp_total_collected = total.get();
        Ok(())
    }
}

impl CollectQuery for MockApi {
    fn dapp_total_rewards(&self, pot: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        assert_eq!(self.rewards_pot, Some(pot.clone().into_string()));
        Ok(NonZeroU128::new(self.dapp_total_rewards))
    }
}

#[cfg(test)]
pub mod dapp;
#[cfg(test)]
pub mod referrer;
