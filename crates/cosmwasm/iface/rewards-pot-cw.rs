use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub dapp: String,
}

#[cw_serde]
pub struct InstantiateResponse {
    pub dapp: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Withdraw any pending rewards
    WithdrawRewards {},
    /// Distribute some collected rewards
    DistributeRewards { recipient: String, amount: Uint128 },
}

#[cw_serde]
#[derive(cosmwasm_schema::QueryResponses)]
pub enum QueryMsg {
    #[returns(TotalRewardsResponse)]
    TotalRewards {},
    /// The dApp for which the pot was created
    #[returns(DappResponse)]
    Dapp {},
    #[returns(AdminResponse)]
    Admin {},
}

#[cw_serde]
pub struct TotalRewardsResponse {
    /// The total amount of rewards received
    pub total: Uint128,
}

#[cw_serde]
pub struct DappResponse {
    /// The dApp address for which the pot was created
    pub dapp: String,
}

#[cw_serde]
pub struct AdminResponse {
    /// The rewards pot admin address
    pub admin: String,
}
