#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use archway_bindings::{ArchwayMsg, ArchwayQuery};
use cosmwasm_std::{Deps as CwDeps, DepsMut as CwDepsMut};

pub type Deps<'a> = CwDeps<'a, ArchwayQuery>;
pub type DepsMut<'a> = CwDepsMut<'a, ArchwayQuery>;
pub type CustomMsg = ArchwayMsg;

pub mod hub;
pub mod rewards_pot;
