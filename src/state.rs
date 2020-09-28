use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use cosmwasm_std::{CanonicalAddr, Storage, Uint128};

pub static CONFIG_KEY: &[u8] = b"config";
pub static USER_CRED_KEY: &[u8] = b"user_cred";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_cred: u64,
    pub total_users: u64,
    pub denom: String,
    pub owner: CanonicalAddr,
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
    pub total_allocated: u64,  // total allocated
    pub allocations: Vec<(String, Allocation)>,  // Maps distribution uuid to allocations
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PolicyType {
    Balanced,
    Immediate,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Allocation {
    pub policy: PolicyType,
    pub amount: u64,
}

pub fn user_cred<S: Storage>(storage: &mut S) -> Bucket<S, UserCred> {
    bucket(USER_CRED_KEY, storage)
}

pub fn user_cred_read<S: Storage>(storage: &S) -> ReadonlyBucket<S, UserCred> {
    bucket_read(USER_CRED_KEY, storage)
}
