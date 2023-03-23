use cosmwasm_std::{Attribute, CosmosMsg, Event, ReplyOn, Response as CwResponse, SubMsg, WasmMsg};
use serde::{de::DeserializeOwned, Serialize};

use referrals_archway_drivers::CustomMsg;

type Response = CwResponse<CustomMsg>;

pub mod hub;
pub mod rewards_pot;

#[derive(Serialize)]
pub enum DisplayWasmMsg<W = ()> {
    Execute { contract_addr: String, msg: W },
    Instantiate { code_id: u64, msg: W, label: String },
}

#[derive(Serialize)]
pub enum DisplayCosmosMsg<W = ()> {
    Std(CosmosMsg<CustomMsg>),
    Wasm(DisplayWasmMsg<W>),
}

#[derive(Serialize)]
pub struct DisplaySubMsg<W = ()> {
    pub id: u64,
    pub msg: DisplayCosmosMsg<W>,
    pub reply_on: ReplyOn,
}

#[derive(Serialize)]
pub struct DisplayResponse<D = (), W = ()> {
    pub data: Option<D>,
    pub messages: Vec<DisplaySubMsg<W>>,
    pub attributes: Vec<Attribute>,
    pub events: Vec<Event>,
}

impl<D, W> From<Response> for DisplayResponse<D, W>
where
    D: DeserializeOwned,
    W: DeserializeOwned,
{
    fn from(response: Response) -> Self {
        let CwResponse {
            messages,
            attributes,
            events,
            data,
            ..
        } = response;

        Self {
            data: data
                .map(|b| cosmwasm_std::from_binary(&b))
                .transpose()
                .unwrap(),
            messages: messages.into_iter().map(DisplaySubMsg::from).collect(),
            attributes,
            events,
        }
    }
}

impl<W> From<SubMsg<CustomMsg>> for DisplaySubMsg<W>
where
    W: DeserializeOwned,
{
    fn from(value: SubMsg<CustomMsg>) -> Self {
        let SubMsg {
            id, msg, reply_on, ..
        } = value;

        Self {
            id,
            msg: DisplayCosmosMsg::from(msg),
            reply_on,
        }
    }
}

impl<W> From<CosmosMsg<CustomMsg>> for DisplayCosmosMsg<W>
where
    W: DeserializeOwned,
{
    fn from(value: CosmosMsg<CustomMsg>) -> Self {
        match value {
            CosmosMsg::Wasm(w) => DisplayCosmosMsg::Wasm(DisplayWasmMsg::from(w)),
            m => Self::Std(m),
        }
    }
}
impl<W> From<WasmMsg> for DisplayWasmMsg<W>
where
    W: DeserializeOwned,
{
    fn from(value: WasmMsg) -> Self {
        match value {
            WasmMsg::Execute {
                contract_addr, msg, ..
            } => DisplayWasmMsg::Execute {
                contract_addr,
                msg: cosmwasm_std::from_binary(&msg).unwrap(),
            },
            WasmMsg::Instantiate {
                code_id,
                msg,
                label,
                ..
            } => DisplayWasmMsg::Instantiate {
                code_id,
                msg: cosmwasm_std::from_binary(&msg).unwrap(),
                label,
            },
            m => panic!("not handling {m:?}"),
        }
    }
}
