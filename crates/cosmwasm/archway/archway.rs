#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use cosmwasm_std::{Addr, CosmosMsg, Response, StdError, SubMsg, Uint128, WasmMsg};

use archway_bindings::ArchwayMsg;
use referrals_cw::ExecuteMsg;

pub trait ResponseExt {
    type SetDappFeeCustom;
    type RecordReferralCustom;

    /// Start the activation process
    fn activate_dapp_referrals(self) -> Activation;

    /// Start the dApp fee setting process
    fn set_dapp_fee(self) -> SetDappFee<Self::SetDappFeeCustom>;

    /// Start the referral recording process
    fn record_referral(self) -> RecordReferral<Self::RecordReferralCustom>;
}

// compile-time-checked-as-much-as-possible message builders

pub struct HubMsg<Msg, CustomMsg, ReferralsHubAddr = ()> {
    msg: Msg,
    referrals_hub_addr: ReferralsHubAddr,
    response: Response<CustomMsg>,
}

pub struct Activate<Name = (), Percent = (), Collector = ()> {
    name: Name,
    percent: Percent,
    collector: Collector,
}

pub struct DappFee<Dapp = (), Fee = ()> {
    dapp: Dapp,
    fee: Fee,
}

pub struct Referral<Code = ()> {
    code: Code,
}

pub type Activation = HubMsg<Activate, ArchwayMsg>;
pub type SetDappFee<C = ()> = HubMsg<DappFee, C>;
pub type RecordReferral<C = ()> = HubMsg<Referral, C>;

impl ResponseExt for Response<ArchwayMsg> {
    type SetDappFeeCustom = ArchwayMsg;
    type RecordReferralCustom = ArchwayMsg;

    fn activate_dapp_referrals(self) -> Activation {
        Activation {
            msg: Activate {
                name: (),
                percent: (),
                collector: (),
            },
            referrals_hub_addr: (),
            response: self,
        }
    }

    fn set_dapp_fee(self) -> SetDappFee<Self::SetDappFeeCustom> {
        SetDappFee {
            msg: DappFee { dapp: (), fee: () },
            referrals_hub_addr: (),
            response: self,
        }
    }

    fn record_referral(self) -> RecordReferral<Self::RecordReferralCustom> {
        RecordReferral {
            msg: Referral { code: () },
            referrals_hub_addr: (),
            response: self,
        }
    }
}

fn to_archway_response(old: Response) -> Response<ArchwayMsg> {
    let mut new = Response::default();
    new.messages = old
        .messages
        .into_iter()
        .map(|msg| SubMsg {
            id: msg.id,
            msg: match msg.msg {
                CosmosMsg::Bank(msg) => CosmosMsg::Bank(msg),
                CosmosMsg::Stargate { type_url, value } => CosmosMsg::Stargate { type_url, value },
                CosmosMsg::Ibc(msg) => CosmosMsg::Ibc(msg),
                CosmosMsg::Wasm(msg) => CosmosMsg::Wasm(msg),
                CosmosMsg::Gov(msg) => CosmosMsg::Gov(msg),
                m => panic!("unhandled msg not converted to archway msg: {m:?}"),
            },
            gas_limit: msg.gas_limit,
            reply_on: msg.reply_on,
        })
        .collect();
    new.attributes = old.attributes;
    new.events = old.events;
    new.data = old.data;
    new
}

impl ResponseExt for Response {
    type SetDappFeeCustom = cosmwasm_std::Empty;
    type RecordReferralCustom = cosmwasm_std::Empty;

    fn activate_dapp_referrals(self) -> Activation {
        Activation {
            msg: Activate {
                name: (),
                percent: (),
                collector: (),
            },
            referrals_hub_addr: (),
            response: to_archway_response(self),
        }
    }

    fn set_dapp_fee(self) -> SetDappFee<Self::RecordReferralCustom> {
        SetDappFee {
            msg: DappFee { dapp: (), fee: () },
            referrals_hub_addr: (),
            response: self,
        }
    }

    fn record_referral(self) -> RecordReferral<Self::RecordReferralCustom> {
        RecordReferral {
            msg: Referral { code: () },
            referrals_hub_addr: (),
            response: self,
        }
    }
}

impl<Msg, Custom> HubMsg<Msg, Custom> {
    /// The address of the Referral Hub
    pub fn referral_hub(self, addr: Addr) -> HubMsg<Msg, Custom, Addr> {
        HubMsg {
            msg: self.msg,
            referrals_hub_addr: addr,
            response: self.response,
        }
    }
}

impl<Custom, Addr, Percent, Collector> HubMsg<Activate<(), Percent, Collector>, Custom, Addr> {
    /// The name of the dApp registering
    pub fn dapp_name(
        self,
        name: impl Into<String>,
    ) -> HubMsg<Activate<String, Percent, Collector>, Custom, Addr> {
        HubMsg {
            msg: Activate {
                name: name.into(),
                percent: self.msg.percent,
                collector: self.msg.collector,
            },
            referrals_hub_addr: self.referrals_hub_addr,
            response: self.response,
        }
    }
}

impl<Custom, Addr, Name, Collector> HubMsg<Activate<Name, (), Collector>, Custom, Addr> {
    /// The percent of contract premiums to give referrers
    pub fn referrer_percent(
        self,
        percent: u8,
    ) -> HubMsg<Activate<Name, u8, Collector>, Custom, Addr> {
        HubMsg {
            msg: Activate {
                name: self.msg.name,
                percent,
                collector: self.msg.collector,
            },
            referrals_hub_addr: self.referrals_hub_addr,
            response: self.response,
        }
    }
}

impl<Custom, Addr, Name, Percent> HubMsg<Activate<Name, Percent, ()>, Custom, Addr> {
    /// The address of the authorised remaining dApp rewards collector
    pub fn collector(self, collector: Addr) -> HubMsg<Activate<Name, Percent, Addr>, Custom, Addr> {
        HubMsg {
            msg: Activate {
                name: self.msg.name,
                percent: self.msg.percent,
                collector,
            },
            referrals_hub_addr: self.referrals_hub_addr,
            response: self.response,
        }
    }
}

impl HubMsg<Activate<String, u8, Addr>, ArchwayMsg, Addr> {
    /// Add the required registration messages to the response.
    /// NOTE: This will transfer rewards admin rights to the Hub.
    /// Either the dApp or the nominated collector can re-gain these rights by de-registering.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The given percent is not in the range 1-100
    /// - There is an issue with `cosmwasm_std` serialization
    pub fn done(self) -> Result<Response<ArchwayMsg>, StdError> {
        if !(1..=100).contains(&self.msg.percent) {
            return Err(StdError::generic_err(
                "Invalid referrer percent - must be in the range 1 - 100",
            ));
        }

        let register = cosmwasm_std::to_binary(&ExecuteMsg::ActivateDapp {
            name: self.msg.name,
            percent: self.msg.percent,
            collector: self.msg.collector.into_string(),
        })?;

        Ok(self
            .response
            .add_message(ArchwayMsg::UpdateContractMetadata {
                contract_address: None, // set self
                owner_address: Some(self.referrals_hub_addr.to_string()),
                rewards_address: None,
            })
            .add_message(WasmMsg::Execute {
                contract_addr: self.referrals_hub_addr.into_string(),
                msg: register,
                funds: vec![],
            }))
    }
}

impl<Custom, HubAddr, Fee> HubMsg<DappFee<(), Fee>, Custom, HubAddr> {
    /// The dApp to set the fee for
    pub fn dapp(self, dapp: Addr) -> HubMsg<DappFee<Addr, Fee>, Custom, HubAddr> {
        HubMsg {
            msg: DappFee {
                dapp,
                fee: self.msg.fee,
            },
            referrals_hub_addr: self.referrals_hub_addr,
            response: self.response,
        }
    }
}

impl<Custom, Addr, Dapp> HubMsg<DappFee<Dapp>, Custom, Addr> {
    /// The new fee amount
    pub fn fee(self, fee: impl Into<Uint128>) -> HubMsg<DappFee<Dapp, Uint128>, Custom, Addr> {
        HubMsg {
            msg: DappFee {
                dapp: self.msg.dapp,
                fee: fee.into(),
            },
            referrals_hub_addr: self.referrals_hub_addr,
            response: self.response,
        }
    }
}

impl<Custom> HubMsg<DappFee<Addr, Uint128>, Custom, Addr> {
    /// Add the required 'set fee' messages to the response.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue serializing the messages.
    pub fn done(self) -> Result<Response<Custom>, StdError> {
        let set_fee = cosmwasm_std::to_binary(&ExecuteMsg::SetDappFee {
            fee: self.msg.fee,
            dapp: self.msg.dapp.into_string(),
        })?;

        Ok(self.response.add_message(WasmMsg::Execute {
            contract_addr: self.referrals_hub_addr.into_string(),
            msg: set_fee,
            funds: vec![],
        }))
    }
}

impl<Custom, Addr> HubMsg<Referral, Custom, Addr> {
    /// The referral code
    pub fn referral_code(self, code: u64) -> HubMsg<Referral<u64>, Custom, Addr> {
        HubMsg {
            msg: Referral { code },
            referrals_hub_addr: self.referrals_hub_addr,
            response: self.response,
        }
    }
}

impl<Custom> HubMsg<Referral<u64>, Custom, Addr> {
    /// Add the required 'record referral' messages to the response.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue serializing the messages.
    pub fn done(self) -> Result<Response<Custom>, StdError> {
        let record_referral = cosmwasm_std::to_binary(&ExecuteMsg::RecordReferral {
            code: self.msg.code,
        })?;

        Ok(self.response.add_message(WasmMsg::Execute {
            contract_addr: self.referrals_hub_addr.into_string(),
            msg: record_referral,
            funds: vec![],
        }))
    }
}
