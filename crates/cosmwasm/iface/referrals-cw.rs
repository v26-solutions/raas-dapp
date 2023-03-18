#![deny(clippy::all)]
#![warn(clippy::pedantic)]

#[path = "rewards-pot-cw.rs"]
pub mod rewards_pot;

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Rewards pot contract code ID
    pub rewards_pot_code_id: u64,
}

#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    /// Register as a referrer.
    /// Responds with `ReferralCodeResponse`
    RegisterReferrer {},
    /// Register as a dApp
    /// Rewards admin rights must be transferred prior to issuing
    RegisterDapp {
        /// The name of the dApp
        name: String,
        /// Percent of flat-fee rewards to give referrers, 1-100
        percent: u8,
        /// Address of nominated rewards collector
        collector: String,
    },
    /// De-register a dApp
    DeregisterDapp {
        /// dApp address to de-register
        dapp: String,
        /// Address of nominated rewards admin
        rewards_admin: String,
        /// Address of nominated rewards recipient
        rewards_recipient: String,
    },
    /// Record a referral
    RecordReferral {
        /// Referral code of referrer
        code: u64,
    },
    /// Collect referrer earnings
    CollectReferrer {
        /// Referral code to collect on behalf of
        code: u64,
        /// dApp address to collect earnings from
        dapp: String,
    },
    /// Collect a dApps remaining rewards
    CollectDapp {
        /// dApp address to collect rewards on behalf of
        dapp: String,
    },
    /// Transfer the ownership of a referral code
    TransferOwnership {
        /// Referral code to transfer ownership of
        code: u64,
        /// The address of the new owner
        owner: String,
    },
    // Configure a registered dApp
    ConfigureDapp {
        /// dApp address to configure
        dapp: String,
        /// Set the a new percentage of fees paid to referrers
        percent: Option<u8>,
        /// Set a new collector address
        collector: Option<String>,
        /// Set a repository URL
        repo_url: Option<String>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct ReferralCodeResponse {
    /// Newly registered referral code
    pub code: u64,
}

#[cosmwasm_schema::cw_serde]
pub enum QueryMsg {}
