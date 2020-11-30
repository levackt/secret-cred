use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{CanonicalAddr, HumanAddr, Uint128};
use crate::state::{PolicyType, ContractInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub token_contract: ContractInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Allocate {
        allocation_id: String,
        amount: Uint128,
        cred_id: String,
        policy_type: PolicyType,
    },
    RegisterUser {
        cred_id: String,
        scrt_address: HumanAddr,
        alias: Option<String>,
    },
//todo handle deregister and update?
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    GetTotalAllocated { cred_id: String},
    IsCredRegistered { cred_id: String},
    IsAllocated { cred_id: String, allocation_id: String },
    GetUserCred { cred_id: String},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TotalAllocatedResponse {
    pub total_allocated: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CredRegisteredResponse {
    pub registered: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CredAllocatedResponse {
    pub allocated: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserCredResponse {
    pub scrt_address: CanonicalAddr,
    pub total_allocated: Uint128,
}
