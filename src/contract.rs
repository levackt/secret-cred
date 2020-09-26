use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage,
};

use crate::msg::{TotalAllocatedResponse, UserRegisteredResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, user_cred, user_cred_read, State};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        totalCred: 0,
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
        HandleMsg::Allocate { id, amount} => try_allocate(deps, env, id, amount),
        HandleMsg::RegisterUser { id } => try_register_user(deps, env, id),
    }
}

pub fn try_allocate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    id: String,
    amount: u64,
) -> StdResult<HandleResponse> {
    //todo verify the id has been registered
    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
    config(&mut deps.storage).update(|mut state| {
        if sender_address_raw != state.owner {
            return Err(StdError::Unauthorized { backtrace: None });
        }
        state.totalCred += amount;
        Ok(state)
    })?;
    //todo update users balance

    Ok(HandleResponse::default())
}

pub fn try_register_user<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    id: String,
) -> StdResult<HandleResponse> {
    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
    let state = config_read(&mut deps.storage).load()?;
    config(&mut deps.storage).update(|mut state| {
        if sender_address_raw != state.owner {
            return Err(StdError::Unauthorized { backtrace: None });
        }
        // todo register user
        Ok(state)
    })?;
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&config_read(&deps.storage).load()?),
        QueryMsg::GetTotalAllocated { user_id } => to_binary(
            &query_total_allocated(deps, user_id)?),
        QueryMsg::IsUserRegistered { user_id } => to_binary(&query_user_registered(deps, user_id)?),
    }
}

fn query_user_registered<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>, id: String) -> StdResult<UserRegisteredResponse> {
    let key = &id.to_string();
    let registered = match user_cred_read(&deps.storage).may_load(key.as_bytes())? {
        Some(cred) => Some(true),
        None => Some(false),
    }.unwrap();

    Ok(UserRegisteredResponse { registered: registered })
}

fn query_total_allocated<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>, id: String) -> StdResult<TotalAllocatedResponse> {

    let key = &id.to_string();
    let cred = match user_cred_read(&deps.storage).may_load(key.as_bytes())? {
        Some(cred) => Some(cred),
        None => return Err(StdError::generic_err("User is not registered")),
    }.unwrap();

    Ok(TotalAllocatedResponse { total_allocated: cred.total_allocated })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};
    pub const DENOM: &str = "uscrt";
    const TEST_CREATOR: &str = "creator";

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg { denom: String::from(DENOM) };
        let env = mock_env(TEST_CREATOR, &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::Config {}).unwrap();
        let value: State = from_binary(&res).unwrap();
        assert_eq!(17, value.count);


        let state = config_read(&mut deps.storage).load().unwrap();
        assert_eq!(
            state,
            State {
                totalCred: 0,
                denom: String::from(DENOM),
                owner: deps
                    .api
                    .canonical_address(&HumanAddr::from(TEST_CREATOR))
                    .unwrap(),
            }
        );
    }
    //
    // #[test]
    // fn increment() {
    //     let mut deps = mock_dependencies(20, &coins(2, "token"));
    //
    //     let msg = InitMsg { count: 17 };
    //     let env = mock_env("creator", &coins(2, "token"));
    //     let _res = init(&mut deps, env, msg).unwrap();
    //
    //     // beneficiary can release it
    //     let env = mock_env("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::Increment {};
    //     let _res = handle(&mut deps, env, msg).unwrap();
    //
    //     // should increase counter by 1
    //     let res = query(&deps, QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }
    //
    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies(20, &coins(2, "token"));
    //
    //     let msg = InitMsg { count: 17 };
    //     let env = mock_env("creator", &coins(2, "token"));
    //     let _res = init(&mut deps, env, msg).unwrap();
    //
    //     // not just anyone can reset the counter
    //     let unauth_env = mock_env("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::Reset { count: 5 };
    //     let res = handle(&mut deps, unauth_env, msg);
    //     match res {
    //         Err(StdError::Unauthorized { .. }) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }
    //
    //     // only the original creator can reset the counter
    //     let auth_env = mock_env("creator", &coins(2, "token"));
    //     let msg = HandleMsg::Reset { count: 5 };
    //     let _res = handle(&mut deps, auth_env, msg).unwrap();
    //
    //     // should now be 5
    //     let res = query(&deps, QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }
}
