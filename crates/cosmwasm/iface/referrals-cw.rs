#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use cosmwasm_schema::{cw_serde, schemars::JsonSchema, serde::Deserialize, serde::Serialize};
use cosmwasm_std::Uint128;

#[path = "rewards-pot-cw.rs"]
pub mod rewards_pot;

#[cw_serde]
pub struct InstantiateMsg {
    /// Rewards pot contract code ID
    pub rewards_pot_code_id: u64,
    /// Contract premium amount
    pub contract_premium: Uint128,
}

#[derive(Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(crate = "::cosmwasm_schema::serde")]
#[schemars(crate = "::cosmwasm_schema::schemars")]
pub struct WithReferralCode<Msg> {
    /// Referral code of sender
    pub referral_code: Option<u64>,
    /// Contract Execution Msg
    // NOTE: Requires custom Deserialize impl.
    // See: https://github.com/CosmWasm/serde-json-wasm/issues/43
    #[serde(flatten)]
    pub msg: Msg,
}

// NOTE: Cannot use `#[cw_serde]` due to it's use of `#[serde(deny_unknown_fields)]`.
// Incompatible with `flatten`: https://serde.rs/container-attrs.html#deny_unknown_fields
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
#[schemars(crate = "::cosmwasm_schema::schemars")]
pub enum ExecuteMsg {
    /// Register as a referrer.
    /// Responds with `ReferralCodeResponse`
    RegisterReferrer {},
    /// Activate as a dApp
    /// Rewards admin rights must be transferred prior to issuing
    ActivateDapp {
        /// The name of the dApp
        name: String,
        /// Percent of flat-fee rewards to give referrers, 1-100
        percent: u8,
        /// Address of nominated rewards collector
        collector: String,
    },
    /// De-activate a dApp
    DeactivateDapp {
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
pub struct ReferralCodeResponse {
    /// Newly registered referral code
    pub code: u64,
}

#[cw_serde]
#[derive(cosmwasm_schema::QueryResponses)]
pub enum QueryMsg {
    #[returns(TotalDappsResponse)]
    TotalDapps {},
    #[returns(DappResponse)]
    Dapp { dapp: String },
    #[returns(AllDappsResponse)]
    AllDapps {
        start: Option<u64>,
        limit: Option<u64>,
    },
    #[returns(ReferralCodeResponse)]
    RefferalCode { referrer: String },
}

#[cw_serde]
pub struct TotalDappsResponse {
    /// Total number of dApps ever activated
    pub total: u64,
}

#[cw_serde]
pub struct DappResponse {
    /// Address of the dApp
    pub address: String,
    /// Active status
    pub active: bool,
    /// Name of the dApp (if Active)
    pub name: Option<String>,
    /// Percent of fee shared with referrers
    pub percent: u8,
    /// Repo URL if set
    pub repo_url: Option<String>,
    /// Fee amount if set
    pub fee: Option<Uint128>,
    /// Total invocations by all referrers
    pub total_invocations: u64,
    /// Number of discrete referrers interacting with the dApp
    pub discrete_referrers: u64,
    /// Total contributions made to referrers
    pub total_contributions: Uint128,
    /// Total rewards earned by dApp
    pub total_rewards: Uint128,
}

#[cw_serde]
pub struct AllDappsResponse {
    /// All the dApp's requested
    pub dapps: Vec<DappResponse>,
}

impl From<ExecuteMsg> for WithReferralCode<ExecuteMsg> {
    fn from(msg: ExecuteMsg) -> Self {
        Self {
            referral_code: None,
            msg,
        }
    }
}

// Custom `Deserialize` required for flattened msg in `WithReferralCode` wrapper
impl<'de, Msg> Deserialize<'de> for WithReferralCode<Msg>
where
    Msg: Deserialize<'de>,
{
    #[allow(clippy::too_many_lines)] // pedantic paperclip smh
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: cosmwasm_schema::serde::Deserializer<'de>,
    {
        use std::fmt;
        use std::marker::PhantomData;

        use cosmwasm_schema::serde;
        use serde::de::{self, Deserializer, MapAccess, Visitor};
        use serde_cw_value::Value;

        #[derive(Debug)]
        enum Field {
            ReferralCode,
            Msg(Value),
        }

        const FIELDS: &[&str] = &["referral_code"];

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`referral_code`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "referral_code" => Ok(Field::ReferralCode),
                            _ => Ok(Field::Msg(Value::String(v.to_owned()))),
                        }
                    }

                    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "referral_code" => Ok(Field::ReferralCode),
                            _ => Ok(Field::Msg(Value::String(v.to_owned()))),
                        }
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            b"referral_code" => Ok(Field::ReferralCode),
                            _ => Ok(Field::Msg(Value::Bytes(v.to_owned()))),
                        }
                    }

                    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v.as_slice() {
                            b"referral_code" => Ok(Field::ReferralCode),
                            _ => Ok(Field::Msg(Value::Bytes(v))),
                        }
                    }

                    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            b"referral_code" => Ok(Field::ReferralCode),
                            _ => Ok(Field::Msg(Value::Bytes(v.to_owned()))),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct WithReferalCodeVisitor<Msg> {
            _m: PhantomData<Msg>,
        }

        impl<'de, Msg> Visitor<'de> for WithReferalCodeVisitor<Msg>
        where
            Msg: Deserialize<'de>,
        {
            type Value = WithReferralCode<Msg>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct WithReferralCode")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut referral_code = None;
                let mut msg = vec![];

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ReferralCode => {
                            if referral_code.is_some() {
                                return Err(de::Error::duplicate_field("referral_code"));
                            }
                            referral_code = map.next_value()?;
                        }
                        Field::Msg(key) => {
                            let value = map.next_value()?;
                            msg.push((key, value));
                        }
                    }
                }

                Ok(WithReferralCode {
                    referral_code,
                    msg: Msg::deserialize(Value::Map(msg.into_iter().collect()))
                        .map_err(|err| de::Error::custom(err.to_string()))?,
                })
            }
        }

        deserializer.deserialize_struct(
            "WithReferralCode",
            FIELDS,
            WithReferalCodeVisitor { _m: PhantomData },
        )
    }
}
