use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError, StdResult, Storage, HumanAddr};

use crate::msg::{TotalAllocatedResponse, CredRegisteredResponse, HandleMsg, InitMsg, QueryMsg, UserCredResponse};
use crate::state::{config, config_read, user_cred, user_cred_read, State, UserCred, PolicyType, Allocation};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        total_cred: 0,
        total_users: 0,
        denom: msg.denom,
        owner: deps.api.canonical_address(&env.message.sender)?,
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
        HandleMsg::RegisterUser { cred_id, scrt_address } => try_register_user(deps, env, cred_id, &scrt_address),
    }
}

pub fn try_allocate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cred_id: String,
    allocation_id: String,
    amount: u64,
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
        state.total_cred += amount;
        cred.total_allocated += amount;
        cred.allocations.push(allocation);

        user_cred(&mut deps.storage).save(key, &cred)?;
        config(&mut deps.storage).save(&state)?;
    } else {
        return Err(StdError::GenericErr {
            msg: "User not registered".to_string(), backtrace: None })
    }
    Ok(HandleResponse::default())
}

pub fn try_register_user<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cred_id: String,
    scrt_address: &HumanAddr,
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
        total_allocated: 0,
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
    pub const DENOM: &str = "uscrt";
    const TEST_CREATOR: &str = "creator";

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);
        mock_init(&mut deps);

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::Config {}).unwrap();
        let state: State = from_binary(&res).unwrap();

        assert_eq!(
            state,
            State {
                total_cred: 0,
                total_users: 0,
                denom: String::from(DENOM),
                owner: deps
                    .api
                    .canonical_address(&HumanAddr::from(TEST_CREATOR))
                    .unwrap(),
            }
        );
    }

    fn mock_init(mut deps: &mut Extern<MockStorage, MockApi, MockQuerier>) {
        let msg = InitMsg { denom: String::from(DENOM) };
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

    #[test]
    fn register_cred_and_query_works() {
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env(TEST_CREATOR, &coins(1000, "hush money"));
        mock_init(&mut deps);

        let cred_id = "street_cred1".to_string();
        let msg = HandleMsg::RegisterUser {
            cred_id,
            scrt_address: HumanAddr("secret007".to_string())
        };

        let _res = handle(&mut deps, env, msg).expect("contract successfully registers cred");
        assert_registered(&mut deps, "street_cred1", true);

        let owner_raw = deps
            .api
            .canonical_address(&HumanAddr::from(TEST_CREATOR))
            .unwrap();
        assert_config_state(&mut deps, State{
            total_cred: 0,
            total_users: 1,
            denom: "uscrt".to_string(),
            owner: owner_raw,
        });

        assert_cred_balance(&mut deps, UserCred {
            cred_id: "street_cred1".to_string(),
            scrt_address: Default::default(),
            total_allocated: 0,
            allocations: vec![]
        });
    }

    #[test]
    fn allocate_cred_works() {
        let mut deps = mock_dependencies(20, &[]);
        mock_init(&mut deps);
        let env = mock_env(TEST_CREATOR, &coins(1000, "hush money"));

        let cred_id = "street_cred1".to_string();
        let msg = HandleMsg::RegisterUser {
            cred_id,
            scrt_address: HumanAddr("secret007".to_string())
        };

        let _res = handle(&mut deps, env.clone(), msg).expect("contract successfully registers cred");
        assert_registered(&mut deps, "street_cred1", true);

        let cred_id = "street_cred1".to_string();
        let msg = HandleMsg::Allocate {
            allocation_id: "allocation 1".to_string(),
            policy_type: PolicyType::Balanced,
            cred_id,
            amount: 100
        };

        let owner_raw = deps
            .api
            .canonical_address(&HumanAddr::from(TEST_CREATOR))
            .unwrap();

        let _res = handle(&mut deps, env.clone(), msg).expect("contract successfully allocates cred");

        assert_config_state(&mut deps, State{
            total_cred: 100,
            total_users: 1,
            denom: "uscrt".to_string(),
            owner: owner_raw,
        });

        assert_cred_balance(&mut deps, UserCred {
            cred_id: "street_cred1".to_string(),
            scrt_address: Default::default(),
            total_allocated: 100,
            allocations: vec![]
        })
    }

    #[test]
    fn register_twice_fails() {
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env(TEST_CREATOR, &coins(1000, "hush money"));
        mock_init(&mut deps);

        let cred_id = "street_cred1".to_string();
        let msg = HandleMsg::RegisterUser {
            cred_id,
            scrt_address: HumanAddr("secret007".to_string())
        };

        let _res = handle(&mut deps, env.clone(), msg.clone()).expect("contract successfully registers cred");
        assert_registered(&mut deps, "street_cred1", true);
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
