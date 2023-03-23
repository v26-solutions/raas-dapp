use std::marker::PhantomData;

use cosmwasm_std::{BankMsg, CosmosMsg, Response as CwResponse, SubMsg};

use dbg_pls::DebugPls;
use serde::de::DeserializeOwned;

use referrals_archway_drivers::CustomMsg;

type Response = CwResponse<CustomMsg>;

pub mod hub;
pub mod rewards_pot;

#[derive(Debug)]
pub struct DisplayResponse<D = (), W = ()> {
    response: Response,
    _d: PhantomData<D>,
    _w: PhantomData<W>,
}

#[derive(Debug)]
pub struct DisplayCosmosMsg<W> {
    msg: SubMsg<CustomMsg>,
    _w: PhantomData<W>,
}

impl<D, W> From<Response> for DisplayResponse<D, W> {
    fn from(response: Response) -> Self {
        Self {
            response,
            _d: PhantomData,
            _w: PhantomData,
        }
    }
}

impl<W> From<SubMsg<CustomMsg>> for DisplayCosmosMsg<W> {
    fn from(msg: SubMsg<CustomMsg>) -> Self {
        Self {
            msg,
            _w: PhantomData,
        }
    }
}

impl<W> DebugPls for DisplayCosmosMsg<W>
where
    W: DeserializeOwned + DebugPls,
{
    fn fmt(&self, f: dbg_pls::Formatter<'_>) {
        let reply_on = format!("{:?}", self.msg.reply_on);

        match self.msg.msg.clone() {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => f
                .debug_struct("BankSend")
                .field("to_address", &to_address)
                .field(
                    "amount",
                    &amount
                        .first()
                        .map(|c| format!("{} {}", c.amount.u128(), c.denom)),
                ),
            CosmosMsg::Custom(custom) => match custom {
                CustomMsg::UpdateContractMetadata {
                    contract_address,
                    owner_address,
                    rewards_address,
                } => f
                    .debug_struct("UpdateContractMetadata")
                    .field("contract_address", &contract_address)
                    .field("owner_address", &owner_address)
                    .field("rewards_address", &rewards_address),
                CustomMsg::WithdrawRewards {
                    records_limit,
                    record_ids,
                } => f
                    .debug_struct("WithdrawRewards")
                    .field("records_limit", &records_limit)
                    .field("record_ids", &record_ids),
                CustomMsg::SetFlatFee {
                    contract_address,
                    flat_fee_amount,
                } => f
                    .debug_struct("SetFlatFee")
                    .field("contract_address", &contract_address)
                    .field("flat_fee_amount", &flat_fee_amount.amount.u128())
                    .field("flat_fee_denom", &flat_fee_amount.denom),
            },
            CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr,
                msg,
                funds,
            }) => {
                let msg = cosmwasm_std::from_binary::<W>(&msg).unwrap();
                f.debug_struct("WasmExecute")
                    .field("contract_addr", &contract_addr)
                    .field("msg", &msg)
                    .field(
                        "funds",
                        &funds
                            .first()
                            .map(|c| format!("{} {}", c.amount.u128(), c.denom)),
                    )
            }
            CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Instantiate {
                admin,
                code_id,
                msg,
                funds,
                label,
            }) => {
                let msg = cosmwasm_std::from_binary::<W>(&msg).unwrap();
                f.debug_struct("WasmInstantiate")
                    .field("admin", &admin)
                    .field("code_id", &code_id)
                    .field("msg", &msg)
                    .field(
                        "funds",
                        &funds
                            .first()
                            .map(|c| format!("{} {}", c.amount.u128(), c.denom)),
                    )
                    .field("label", &label)
            }
            _ => panic!("unhandled msg type"),
        }
        .field("reply_on", &reply_on)
        .finish()
    }
}

impl<D, W> DebugPls for DisplayResponse<D, W>
where
    D: DeserializeOwned + DebugPls,
    W: DeserializeOwned + DebugPls,
{
    fn fmt(&self, f: dbg_pls::Formatter<'_>) {
        let Response { messages, data, .. } = self.response.clone();

        let data = data
            .as_ref()
            .map(cosmwasm_std::from_binary::<D>)
            .transpose()
            .unwrap();

        let messages: Vec<DisplayCosmosMsg<W>> =
            messages.into_iter().map(DisplayCosmosMsg::from).collect();

        f.debug_struct("Response")
            .field("data", &data)
            .field("messages", &messages)
            .finish()
    }
}
