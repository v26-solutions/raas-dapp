use referrals_core::hub::{
    MutableReferralStore, ReadonlyDappStore, ReadonlyReferralStore, ReferralCode,
};

use super::*;

impl ReadonlyReferralStore for MockApi {
    fn code_exists(&self, code: ReferralCode) -> Result<bool, Self::Error> {
        Ok(self.referral_code.map_or(false, |c| c == code.to_u64()))
    }

    fn owner_exists(&self, owner: &Id) -> Result<bool, Self::Error> {
        Ok(self
            .referral_code_owner
            .as_ref()
            .map_or(false, |o| o == owner.as_str()))
    }

    fn owner_of(&self, code: ReferralCode) -> Result<Option<Id>, Self::Error> {
        if !self.code_exists(code)? {
            return Ok(None);
        }

        Ok(self.referral_code_owner.clone().map(Id::from))
    }

    fn latest(&self) -> Result<Option<ReferralCode>, Self::Error> {
        Ok(self.latest_referral_code.map(ReferralCode::from))
    }

    fn total_earnings(&self, code: ReferralCode) -> Result<Option<NonZeroU128>, Self::Error> {
        assert!(self.code_exists(code)?);
        Ok(NonZeroU128::new(self.code_total_earnings))
    }

    fn dapp_earnings(
        &self,
        _dapp: &Id,
        code: ReferralCode,
    ) -> Result<Option<NonZeroU128>, Self::Error> {
        assert!(self.code_exists(code)?);
        Ok(NonZeroU128::new(self.code_dapp_earnings))
    }

    fn dapp_contributions(&self, _dapp: &Id) -> Result<Option<NonZeroU128>, Self::Error> {
        Ok(NonZeroU128::new(self.dapp_contributions))
    }
}

impl MutableReferralStore for MockApi {
    fn set_latest(&mut self, code: ReferralCode) -> Result<(), Self::Error> {
        self.latest_referral_code = Some(code.to_u64());
        Ok(())
    }

    fn set_code_owner(&mut self, code: ReferralCode, owner: Id) -> Result<(), Self::Error> {
        self.referral_code = Some(code.to_u64());
        self.referral_code_owner = Some(owner.into_string());
        Ok(())
    }

    fn increment_invocations(&mut self, dapp: &Id, code: ReferralCode) -> Result<(), Self::Error> {
        assert!(self.dapp_exists(dapp)?);
        assert!(self.code_exists(code)?);
        self.dapp_reffered_invocations += 1;
        Ok(())
    }

    fn set_total_earnings(
        &mut self,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        assert!(self.code_exists(code)?);
        self.code_total_earnings = total.get();
        Ok(())
    }

    fn set_dapp_earnings(
        &mut self,
        dapp: &Id,
        code: ReferralCode,
        total: NonZeroU128,
    ) -> Result<(), Self::Error> {
        assert!(self.dapp_exists(dapp)?);
        assert!(self.code_exists(code)?);
        self.code_dapp_earnings = total.get();
        Ok(())
    }

    fn set_dapp_contributions(
        &mut self,
        dapp: &Id,
        contributions: NonZeroU128,
    ) -> Result<(), Self::Error> {
        assert!(self.dapp_exists(dapp)?);
        self.dapp_contributions = contributions.get();
        Ok(())
    }
}

#[cfg(test)]
pub mod record;
#[cfg(test)]
pub mod register;
#[cfg(test)]
pub mod transfer_ownership;
