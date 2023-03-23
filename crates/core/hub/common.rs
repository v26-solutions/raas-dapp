use std::num::NonZeroU128;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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
