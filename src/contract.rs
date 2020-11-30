use cosmwasm_std::{to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, InitResponse, log, Querier, StdError, StdResult, Storage, HumanAddr, Uint128};

use crate::msg::{ CredAllocatedResponse, CredRegisteredResponse, HandleMsg, InitMsg, QueryMsg, UserCredResponse, TotalAllocatedResponse};
use crate::state::{config, config_read, user_cred, user_cred_read, State, UserCred, PolicyType, Allocation};
use crate::tokens::mint;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        total_cred: Uint128::zero(),
        total_users: 0,
        owner: deps.api.canonical_address(&env.message.sender)?,
        token_contract: msg.token_contract
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Allocate { cred_id, allocation_id, amount,  policy_type } => try_allocate(
            deps,
            env,
            cred_id,
            allocation_id,
            amount,
            policy_type
        ),
        HandleMsg::RegisterUser { cred_id, scrt_address, alias } =>
            try_register_user(deps, env, cred_id, &scrt_address, alias),
    }
}

pub fn try_allocate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cred_id: String,
    allocation_id: String,
    amount: Uint128,
    policy_type: PolicyType,
) -> StdResult<HandleResponse> {

    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;

    let allocation = Allocation {
        policy: policy_type,
        amount,
        allocation_id
    };

    let mut state = config(&mut deps.storage).load()?;
    // only owner
    if sender_address_raw != state.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let key = &cred_id.as_bytes();
    if let Some(mut cred) = user_cred(&mut deps.storage).may_load(key)? {

        if cred.allocations.contains(&allocation) {
            return Err(StdError::GenericErr {
                msg: "Already allocated".to_string(), backtrace: None })
        }

        state.total_cred += amount;
        cred.total_allocated += amount;
        cred.allocations.push(allocation);

        user_cred(&mut deps.storage).save(key, &cred)?;
        config(&mut deps.storage).save(&state)?;
        let scrt_addy = deps.api.human_address(&cred.scrt_address)?;

        let mut messages: Vec<CosmosMsg> = vec![];

        messages.push(mint(
            &deps.storage,
            amount,
            scrt_addy,
        )?);
        let res = HandleResponse {
            messages,
            log: vec![
                log("action", "allocate-cred"),
                log("account", env.message.sender.as_str()),
                log("amount", &amount.to_string()),
            ],
            data: None,
        };
        Ok(res)
    } else {
        return Err(StdError::GenericErr {
            msg: "User not registered".to_string(), backtrace: None })
    }
}

pub fn try_register_user<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cred_id: String,
    scrt_address: &HumanAddr,
    alias: Option<String>,
) -> StdResult<HandleResponse> {
    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
    let mut state = config(&mut deps.storage).load()?;
    if sender_address_raw != state.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // user must not exist
    let key = cred_id.as_bytes();
    let registered = match user_cred_read(&deps.storage).may_load(key)? {
        Some(_) => Some(true),
        None => Some(false),
    }.unwrap();
    if registered {
        return Err(StdError::GenericErr {
            msg: "User already exists".to_string(), backtrace: None })
    }

    let scrt_address_raw = deps.api.canonical_address(scrt_address)?;

    let cred = &UserCred{
        cred_id: cred_id.to_string(),
        scrt_address: scrt_address_raw,
        alias,
        total_allocated: Uint128::zero(),
        allocations: vec![]
    };

    user_cred(&mut deps.storage).save(key, &cred)?;

    state.total_users = state.total_users + 1;
    config(&mut deps.storage).save(&state)?;

    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&config_read(&deps.storage).load()?),
        QueryMsg::GetTotalAllocated { cred_id } => to_binary(
            &query_total_allocated(deps, cred_id)?),
        QueryMsg::IsCredRegistered { cred_id } => to_binary(&query_user_registered(deps, cred_id)?),
        QueryMsg::GetUserCred { cred_id } => to_binary(&query_user_cred(deps, cred_id)?),
        QueryMsg::IsAllocated { cred_id, allocation_id } => to_binary(&query_allocated(deps, cred_id, allocation_id)?),
    }
}

fn query_user_registered<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>, id: String) -> StdResult<CredRegisteredResponse> {
    let key = &id.as_bytes();
    let registered = match user_cred_read(&deps.storage).may_load(key)? {
        Some(_) => Some(true),
        None => Some(false),
    }.unwrap();

    Ok(CredRegisteredResponse { registered })
}

fn query_allocated<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>, id: String, allocation_id: String) -> StdResult<CredAllocatedResponse> {
    let key = &id.as_bytes();
    let cred = match user_cred_read(&deps.storage).may_load(key)? {
        Some(cred) => Some(cred),
        None => return Err(StdError::GenericErr { msg: "User does not exist".to_string(), backtrace: None }),
    }.unwrap();

    let allocation = Allocation {
        policy: PolicyType::Balanced,
        amount: Uint128::zero(),
        allocation_id
    };

    Ok(CredAllocatedResponse { allocated: cred.allocations.contains(&allocation) })
}

fn query_user_cred<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>, id: String) -> StdResult<UserCredResponse> {
    let key = &id.as_bytes();
    let cred = match user_cred_read(&deps.storage).may_load(key)? {
        Some(cred) => Some(cred),
        None => return Err(StdError::GenericErr { msg: "User does not exist".to_string(), backtrace: None }),
    }.unwrap();

    Ok(UserCredResponse { scrt_address: cred.scrt_address, total_allocated: cred.total_allocated })
}

fn query_total_allocated<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>, id: String) -> StdResult<TotalAllocatedResponse> {

    let key = &id.as_bytes();
    let cred = match user_cred_read(&deps.storage).may_load(key)? {
        Some(cred) => Some(cred),
        None => return Err(StdError::generic_err("User is not registered")),
    }.unwrap();

    Ok(TotalAllocatedResponse { total_allocated: cred.total_allocated })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{coins, from_binary, StdError};
    use crate::state::{ContractInfo};
    pub const TOKEN_HASH: &str = "foocoinhash";
    
    const TEST_CREATOR: &str = "creator";

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);
        mock_init(&mut deps);

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::Config {}).unwrap();
        let state: State = from_binary(&res).unwrap();
        let token_contract = ContractInfo {
            code_hash: TOKEN_HASH.to_string(),
            address: HumanAddr(TOKEN_HASH.to_string()),
        };

        assert_eq!(
            state,
            State {
                total_cred: Uint128::zero(),
                total_users: 0,
                token_contract,
                owner: deps
                    .api
                    .canonical_address(&HumanAddr::from(TEST_CREATOR))
                    .unwrap(),
            }
        );
    }

    fn mock_init(mut deps: &mut Extern<MockStorage, MockApi, MockQuerier>) {
        let token_contract = ContractInfo {
            code_hash: TOKEN_HASH.to_string(),
            address: HumanAddr(TOKEN_HASH.to_string()),
        };

        let msg = InitMsg { token_contract };
        let env = mock_env(TEST_CREATOR, &coins(1000, "hush money"));

        let _res = init(&mut deps, env, msg).expect("contract successfully handles InitMsg");
    }

    fn assert_registered(
        deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
        cred_id: &str,
        expected: bool,
    ) {
        let res = query(
            &deps,
            QueryMsg::IsCredRegistered {
                cred_id: cred_id.to_string(),
            },
        ).unwrap();

        let value: CredRegisteredResponse = from_binary(&res).unwrap();
        assert_eq!(expected, value.registered);
    }

    fn assert_config_state(deps: &mut Extern<MockStorage, MockApi, MockQuerier>, expected: State) {
        let res = query(&deps, QueryMsg::Config {}).unwrap();
        let value: State = from_binary(&res).unwrap();
        assert_eq!(value, expected);
    }

    fn assert_cred_balance(deps: &mut Extern<MockStorage, MockApi, MockQuerier>, expected: UserCred) {
        let res = query(&deps, QueryMsg::GetUserCred { cred_id: expected.cred_id }).unwrap();
        let value: UserCredResponse = from_binary(&res).unwrap();
        assert_eq!(value.total_allocated, expected.total_allocated);
    }

    fn assert_cred_allocated(deps: &mut Extern<MockStorage, MockApi, MockQuerier>, cred_id: String, allocation_id: String, expected: bool) {
        let res = query(&deps, QueryMsg::IsAllocated { cred_id, allocation_id }).unwrap();
        let value: CredAllocatedResponse = from_binary(&res).unwrap();
        assert_eq!(value.allocated, expected);
    }

    #[test]
    fn register_cred_and_query_works() {
        let token_contract = ContractInfo {
            code_hash: TOKEN_HASH.to_string(),
            address: HumanAddr(TOKEN_HASH.to_string()),
        };
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env(TEST_CREATOR, &coins(1000, "hush money"));
        mock_init(&mut deps);

        let cred_id = "cred1".to_string();
        let msg = HandleMsg::RegisterUser {
            cred_id,
            scrt_address: HumanAddr("secret007".to_string()),
            alias: Some("secret007".to_string()),
        };

        let _res = handle(&mut deps, env, msg).expect("contract successfully registers cred");
        assert_registered(&mut deps, "cred1", true);

        let owner_raw = deps
            .api
            .canonical_address(&HumanAddr::from(TEST_CREATOR))
            .unwrap();
        assert_config_state(&mut deps, State{
            total_cred: Uint128::zero(),
            total_users: 1,
            token_contract,
            owner: owner_raw,
        });

        assert_cred_balance(&mut deps, UserCred {
            cred_id: "cred1".to_string(),
            scrt_address: Default::default(),
            total_allocated: Uint128::zero(),
            allocations: vec![],
            alias: None,
        });
    }

    #[test]
    fn allocate_cred_works() {
        let mut deps = mock_dependencies(20, &[]);
        mock_init(&mut deps);
        let env = mock_env(TEST_CREATOR, &coins(100u128, "foocoin"));

        let cred_id = "cred1".to_string();
        let msg = HandleMsg::RegisterUser {
            cred_id,
            scrt_address: HumanAddr("secret007".to_string()),
            alias: None,
        };

        let _res = handle(&mut deps, env.clone(), msg).expect("contract successfully registers cred");
        assert_registered(&mut deps, "cred1", true);

        let cred_id = "cred1".to_string();
        let msg = HandleMsg::Allocate {
            allocation_id: "allocation 1".to_string(),
            policy_type: PolicyType::Balanced,
            cred_id,
            amount: Uint128::from(100u128)
        };

        let owner_raw = deps
            .api
            .canonical_address(&HumanAddr::from(TEST_CREATOR))
            .unwrap();

        let _res = handle(&mut deps, env.clone(), msg).expect("contract successfully allocates cred");

        let token_contract = ContractInfo {
            code_hash: TOKEN_HASH.to_string(),
            address: HumanAddr(TOKEN_HASH.to_string()),
        };
        assert_config_state(&mut deps, State{
            total_cred: Uint128::from(100u128),
            total_users: 1,
            token_contract,
            owner: owner_raw,
        });

        assert_cred_balance(&mut deps, UserCred {
            cred_id: "cred1".to_string(),
            scrt_address: Default::default(),
            total_allocated: Uint128::from(100u128),
            allocations: vec![],
            alias: None
        });

        assert_cred_allocated(&mut deps, "cred1".to_string(), "allocation 1".to_string(), true);
    }


    #[test]
    fn allocate_cred_twice_works() {
        let mut deps = mock_dependencies(20, &[]);
        mock_init(&mut deps);
        let env = mock_env(TEST_CREATOR, &coins(29416857982094508000u128, "foocoin"));

        let cred_id = "cred1".to_string();
        let msg = HandleMsg::RegisterUser {
            cred_id,
            scrt_address: HumanAddr("secret007".to_string()),
            alias: None,
        };

        let _res = handle(&mut deps, env.clone(), msg).expect("contract successfully registers cred");
        assert_registered(&mut deps, "cred1", true);

        let cred_id = "cred1".to_string();
        let msg = HandleMsg::Allocate {
            allocation_id: "allocation 1".to_string(),
            policy_type: PolicyType::Balanced,
            cred_id: cred_id.clone(),
            amount: Uint128::from(14708428991047254000u128)
        };

        let owner_raw = deps
            .api
            .canonical_address(&HumanAddr::from(TEST_CREATOR))
            .unwrap();

        let _res = handle(&mut deps, env.clone(), msg).expect("contract successfully allocates cred");

        let token_contract = ContractInfo {
            code_hash: TOKEN_HASH.to_string(),
            address: HumanAddr(TOKEN_HASH.to_string()),
        };
        assert_config_state(&mut deps, State{
            total_cred: Uint128::from(14708428991047254000u128),
            total_users: 1,
            token_contract,
            owner: owner_raw.clone(),
        });

        assert_cred_balance(&mut deps, UserCred {
            cred_id: "cred1".to_string(),
            scrt_address: Default::default(),
            total_allocated: Uint128::from(14708428991047254000u128),
            allocations: vec![],
            alias: None
        });


        let msg = HandleMsg::Allocate {
            allocation_id: "allocation 2".to_string(),
            policy_type: PolicyType::Balanced,
            cred_id: cred_id.clone(),
            amount: Uint128::from(29416857982094508000u128)
        };

        let handle_res = handle(&mut deps, env.clone(), msg).expect("contract successfully allocates more cred");

        let _msg = handle_res.messages.get(0).expect("no message");

        let token_contract = ContractInfo {
            code_hash: TOKEN_HASH.to_string(),
            address: HumanAddr(TOKEN_HASH.to_string()),
        };
        assert_config_state(&mut deps, State{
            total_cred: Uint128::from(44125286973141762000u128),
            total_users: 1,
            token_contract,
            owner: owner_raw,
        });

        assert_cred_balance(&mut deps, UserCred {
            cred_id: "cred1".to_string(),
            scrt_address: Default::default(),
            total_allocated: Uint128::from(44125286973141762000u128),
            allocations: vec![],
            alias: None
        })
    }

    #[test]
    fn allocate_cred_duplicate_fails() {
        let mut deps = mock_dependencies(20, &[]);
        mock_init(&mut deps);
        let env = mock_env(TEST_CREATOR, &coins(100u128, "foocoin"));

        let cred_id = "cred1".to_string();
        let msg = HandleMsg::RegisterUser {
            cred_id,
            scrt_address: HumanAddr("secret007".to_string()),
            alias: None,
        };

        let _res = handle(&mut deps, env.clone(), msg).expect("contract successfully registers cred");
        assert_registered(&mut deps, "cred1", true);

        let cred_id = "cred1".to_string();
        let msg = HandleMsg::Allocate {
            allocation_id: "allocation 1".to_string(),
            policy_type: PolicyType::Balanced,
            cred_id,
            amount: Uint128::from(100u128)
        };

        let owner_raw = deps
            .api
            .canonical_address(&HumanAddr::from(TEST_CREATOR))
            .unwrap();

        let _res = handle(&mut deps, env.clone(), msg.clone()).expect("contract successfully allocates cred");

        let token_contract = ContractInfo {
            code_hash: TOKEN_HASH.to_string(),
            address: HumanAddr(TOKEN_HASH.to_string()),
        };
        assert_config_state(&mut deps, State{
            total_cred: Uint128::from(100u128),
            total_users: 1,
            token_contract: token_contract.clone(),
            owner: owner_raw.clone(),
        });

        assert_cred_balance(&mut deps, UserCred {
            cred_id: "cred1".to_string(),
            scrt_address: Default::default(),
            total_allocated: Uint128::from(100u128),
            allocations: vec![],
            alias: None
        });

        let _res = handle(&mut deps, env.clone(), msg);

        match _res {
            Ok(_) => panic!("expected error"),
            Err(StdError::GenericErr { msg, .. }) => {
                assert_eq!(msg, "Already allocated")
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }

        assert_config_state(&mut deps, State{
            total_cred: Uint128::from(100u128),
            total_users: 1,
            token_contract,
            owner: owner_raw,
        });

        assert_cred_balance(&mut deps, UserCred {
            cred_id: "cred1".to_string(),
            scrt_address: Default::default(),
            total_allocated: Uint128::from(100u128),
            allocations: vec![],
            alias: None
        })
    }


    #[test]
    fn register_twice_fails() {
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env(TEST_CREATOR, &coins(1000, "hush money"));
        mock_init(&mut deps);

        let cred_id = "cred1".to_string();
        let msg = HandleMsg::RegisterUser {
            cred_id,
            scrt_address: HumanAddr("secret007".to_string()),
            alias: None,
        };

        let _res = handle(&mut deps, env.clone(), msg.clone()).expect("contract successfully registers cred");
        assert_registered(&mut deps, "cred1", true);
        let _res = handle(&mut deps, env, msg);
        match _res {
            Ok(_) => panic!("expected error"),
            Err(StdError::GenericErr { msg, .. }) => {
                assert_eq!(msg, "User already exists")
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }
}
