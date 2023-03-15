use cosmwasm_std::Uint128;

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {}

#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    /// Withdraw any pending rewards
    WithdrawRewards {},
    /// Distribute some collected rewards
    DistributeRewards { recipient: String, amount: Uint128 },
}

#[cosmwasm_schema::cw_serde]
#[derive(cosmwasm_schema::QueryResponses)]
pub enum QueryMsg {
    /// Total pot rewards
    #[returns(TotalRewardsResponse)]
    TotalRewards {},
}

#[cosmwasm_schema::cw_serde]
pub struct TotalRewardsResponse {
    /// The total amount of rewards received
    total: Uint128,
}
