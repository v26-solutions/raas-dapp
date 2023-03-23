#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use std::error::Error as StdError;

use serde::{Deserialize, Serialize};

pub mod hub;
pub mod rewards_pot;

pub trait FallibleApi {
    type Error: StdError;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
