#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[path = "rewards-pot-cw.rs"]
pub mod rewards_pot;

#[cw_serde]
pub struct InstantiateMsg {
    /// Rewards pot contract code ID
    pub rewards_pot_code_id: u64,
}

#[cw_serde]
#[derive(dbg_pls::DebugPls)]
pub struct WithReferralCode<Msg> {
    /// Referral code of sender
    pub referral_code: Option<u64>,
    /// Contract Execution Msg
    #[serde(flatten)]
    pub msg: Msg,
}

#[cw_serde]
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
    /// Set a dApp's flat fee
    SetDappFee {
        /// dApp address to set fee for
        dapp: String,
        /// Fee amount
        fee: Uint128,
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

#[cw_serde]
#[derive(dbg_pls::DebugPls)]
pub struct ReferralCodeResponse {
    /// Newly registered referral code
    pub code: u64,
}

#[cw_serde]
pub enum QueryMsg {}

impl From<ExecuteMsg> for WithReferralCode<ExecuteMsg> {
    fn from(msg: ExecuteMsg) -> Self {
        Self {
            referral_code: None,
            msg,
        }
    }
}

impl dbg_pls::DebugPls for ExecuteMsg {
    fn fmt(&self, f: dbg_pls::Formatter<'_>) {
        match self {
            ExecuteMsg::RegisterReferrer {} => f.debug_ident("RegisterReferrer"),
            ExecuteMsg::RegisterDapp {
                name,
                percent,
                collector,
            } => f
                .debug_struct("RegisterDapp")
                .field("name", &name)
                .field("percent", &percent)
                .field("collector", collector)
                .finish(),
            ExecuteMsg::DeregisterDapp {
                dapp,
                rewards_admin,
                rewards_recipient,
            } => f
                .debug_struct("DeregisterDapp")
                .field("dapp", &dapp)
                .field("rewards_admin", &rewards_admin)
                .field("rewards_recipient", &rewards_recipient)
                .finish(),
            ExecuteMsg::SetDappFee { dapp, fee } => f
                .debug_struct("SetDappFee")
                .field("dapp", &dapp)
                .field("fee", &fee.u128())
                .finish(),
            ExecuteMsg::RecordReferral { code } => f
                .debug_struct("RecordReferral")
                .field("code", &code)
                .finish(),
            ExecuteMsg::CollectReferrer { code, dapp } => f
                .debug_struct("CollectReferrer")
                .field("code", &code)
                .field("dapp", &dapp)
                .finish(),
            ExecuteMsg::CollectDapp { dapp } => {
                f.debug_struct("CollectDapp").field("dapp", &dapp).finish();
            }
            ExecuteMsg::TransferOwnership { code, owner } => f
                .debug_struct("TransferOwnership")
                .field("code", &code)
                .field("owner", &owner)
                .finish(),
            ExecuteMsg::ConfigureDapp {
                dapp,
                percent,
                collector,
                repo_url,
            } => f
                .debug_struct("ConfigureDapp")
                .field("dapp", &dapp)
                .field("percent", &percent)
                .field("collector", &collector)
                .field("repo_url", &repo_url)
                .finish(),
        }
    }
}
