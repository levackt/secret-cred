use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{CanonicalAddr, HumanAddr};
use crate::state::{PolicyType};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub denom: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Allocate {
        allocation_id: String,
        amount: u64,
        cred_id: String,
        policy_type: PolicyType,
    },
    RegisterUser {
        cred_id: String,
        scrt_address: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    GetTotalAllocated { cred_id: String},
    IsCredRegistered { cred_id: String},
    GetUserCred { cred_id: String},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TotalAllocatedResponse {
    pub total_allocated: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CredRegisteredResponse {
    pub registered: bool,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserCredResponse {
    pub scrt_address: CanonicalAddr,
    pub total_allocated: u64,
}
