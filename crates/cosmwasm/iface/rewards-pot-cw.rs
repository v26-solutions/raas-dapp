use cosmwasm_std::Uint128;

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub dapp: String,
}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateResponse {
    pub dapp: String,
}

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
    #[returns(TotalRewardsResponse)]
    TotalRewards {},
    /// The dApp for which the pot was created
    #[returns(DappResponse)]
    DappRewards {},
    #[returns(AdminResponse)]
    Admin {},
}

#[cosmwasm_schema::cw_serde]
pub struct TotalRewardsResponse {
    /// The total amount of rewards received
    pub total: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct DappResponse {
    /// The dApp address for which the pot was created
    pub dapp: String,
}

#[cosmwasm_schema::cw_serde]
pub struct AdminResponse {
    /// The rewards pot admin address
    pub admin: String,
}
