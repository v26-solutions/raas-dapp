use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
#[derive(dbg_pls::DebugPls)]
pub struct InstantiateMsg {
    pub dapp: String,
}

#[cw_serde]
#[derive(dbg_pls::DebugPls)]
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
#[derive(dbg_pls::DebugPls, cosmwasm_schema::QueryResponses)]
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
#[derive(dbg_pls::DebugPls)]
pub struct DappResponse {
    /// The dApp address for which the pot was created
    pub dapp: String,
}

#[cw_serde]
#[derive(dbg_pls::DebugPls)]
pub struct AdminResponse {
    /// The rewards pot admin address
    pub admin: String,
}

impl dbg_pls::DebugPls for ExecuteMsg {
    fn fmt(&self, f: dbg_pls::Formatter<'_>) {
        match self {
            ExecuteMsg::WithdrawRewards {} => f.debug_ident("WithdrawRewards"),
            ExecuteMsg::DistributeRewards { recipient, amount } => f
                .debug_struct("DistibuteRewards")
                .field("recipient", &recipient)
                .field("amount", &amount.u128())
                .finish(),
        }
    }
}

impl dbg_pls::DebugPls for TotalRewardsResponse {
    fn fmt(&self, f: dbg_pls::Formatter<'_>) {
        f.debug_struct("TotalRewardsResponse")
            .field("total", &self.total.u128())
            .finish();
    }
}
