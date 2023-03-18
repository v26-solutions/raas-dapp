use std::num::NonZeroU128;

use referrals_core::{FallibleApi, Id, NonZeroPercent};

use dbg_pls::DebugPls;

#[derive(DebugPls, Default)]
pub struct MockApi {
    dapp: Option<String>,
    percent: Option<u8>,
    collector: Option<String>,
    rewards_pot: Option<String>,
    rewards_pot_admin: Option<String>,
    rewards_admin: Option<String>,
    current_fee: Option<NonZeroU128>,
    referral_code: Option<u64>,
    referral_code_owner: Option<String>,
    latest_referral_code: Option<u64>,
    dapp_reffered_invocations: u64,
    code_total_earnings: u128,
    code_dapp_earnings: u128,
    dapp_contributions: u128,
    code_total_collected: u128,
    code_dapp_collected: u128,
    dapp_total_collected: u128,
    dapp_total_rewards: u128,
}

#[macro_export]
macro_rules! nzp {
    ($p:literal) => {
        referrals_core::NonZeroPercent::new($p).unwrap()
    };
}

#[macro_export]
macro_rules! nz {
    ($n:literal) => {
        std::num::NonZeroU128::new($n).unwrap()
    };
}

impl MockApi {
    pub fn dapp(mut self, id: &str) -> Self {
        self.dapp = Some(id.into());
        self
    }

    pub fn collector(mut self, id: &str) -> Self {
        self.collector = Some(id.into());
        self
    }

    pub fn rewards_admin(mut self, id: &str) -> Self {
        self.rewards_admin = Some(id.into());
        self
    }

    pub fn rewards_pot(mut self, id: &str) -> Self {
        self.rewards_pot = Some(id.into());
        self
    }

    pub fn rewards_pot_admin(mut self, id: &str) -> Self {
        self.rewards_pot_admin = Some(id.into());
        self
    }

    pub fn current_fee(mut self, fee: NonZeroU128) -> Self {
        self.current_fee = Some(fee);
        self
    }

    pub fn referral_code(mut self, code: u64) -> Self {
        self.referral_code = Some(code);
        self
    }

    pub fn referral_code_owner(mut self, id: &str) -> Self {
        self.referral_code_owner = Some(id.into());
        self
    }

    pub fn dapp_total_rewards(mut self, total: u128) -> Self {
        self.dapp_total_rewards = total;
        self
    }

    pub fn set_dapp_total_rewards(&mut self, total: u128) -> &mut Self {
        self.dapp_total_rewards = total;
        self
    }

    pub fn set_current_fee(&mut self, fee: NonZeroU128) -> &mut Self {
        self.current_fee = Some(fee);
        self
    }
}

impl FallibleApi for MockApi {
    type Error = std::convert::Infallible;
}

pub mod collect;
pub mod dapp;
#[cfg(test)]
pub mod exec;
pub mod referral;
