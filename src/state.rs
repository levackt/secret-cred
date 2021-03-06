use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use cosmwasm_std::{HumanAddr, CanonicalAddr, Storage, Uint128};

pub static CONFIG_KEY: &[u8] = b"config";
pub static USER_CRED_KEY: &[u8] = b"user_cred";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_cred: Uint128,
    pub total_users: u64,
    pub owner: CanonicalAddr,
    pub token_contract: ContractInfo,
}

// struct containing token contract info
// hash: String -- code hash of the SNIP-20 token contract
// address: HumanAddr -- address of the SNIP-20 token contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfo {
    pub code_hash: String,
    pub address: HumanAddr,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserCred {
    pub cred_id: String,  // ID on source cred
    pub scrt_address: CanonicalAddr,  //  public address of registered user
    pub total_allocated: Uint128,  // total allocated
    pub allocations: Vec<Allocation>,
    pub alias: Option<String>,  // Optionally an alias
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PolicyType {
    Balanced,
    Immediate,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Allocation {
    pub policy: PolicyType,
    pub amount: Uint128,
    pub allocation_id: String,
}

impl PartialEq for Allocation {
    fn eq(&self, other: &Self) -> bool {
        self.allocation_id == other.allocation_id
    }
}

pub fn user_cred<S: Storage>(storage: &mut S) -> Bucket<S, UserCred> {
    bucket(USER_CRED_KEY, storage)
}

pub fn user_cred_read<S: Storage>(storage: &S) -> ReadonlyBucket<S, UserCred> {
    bucket_read(USER_CRED_KEY, storage)
}
