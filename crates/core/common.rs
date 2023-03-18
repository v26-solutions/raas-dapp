use std::error::Error as StdError;
use std::num::NonZeroU128;

pub trait FallibleApi {
    type Error: StdError;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Id(String);

impl Id {
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.as_ref()
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
