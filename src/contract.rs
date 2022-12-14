//use std::task::Poll;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
//use cw2::{CONTRACT, set_contract_version};
use cw2::{set_contract_version};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetPollResponse, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, POLL, SPoll};


const CONTRACT_NAME: &str = "crates.io:cw-step1";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let validated_address = deps.api.addr_validate(&msg.admin_address)?;

    let config = Config{ admin_address: validated_address };

    CONFIG.save(deps.storage,&config)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {

    match msg  {
        ExecuteMsg::CreatePollMsg {poll_name} =>execute_create_poll(deps, env, info, poll_name),
        ExecuteMsg::VotePollMsg {poll_name, choice } =>execute_vote_poll(deps, env, info, poll_name, choice)
    }

}

fn execute_create_poll (deps: DepsMut, _env: Env, _info:MessageInfo, poll_name:String) -> Result<Response, ContractError>{

    if POLL.has(deps.storage, poll_name.clone()) {
        return Err(ContractError::CustomError { val: "Poll Created".to_string() });
    }
    let my_poll = SPoll{
        poll_name: poll_name.clone(),
        yes_votes: 0,
        no_votes: 0
    };
    POLL.save(deps.storage, poll_name, &my_poll)?;
    Ok(Response::new().add_attribute("action", "poll_created"))
}

fn execute_vote_poll (deps: DepsMut, env: Env, info: MessageInfo, poll_name: String, choice:String) -> Result<Response, ContractError>{

    if !POLL.has(deps.storage, poll_name.clone()) {
        return Err(ContractError::CustomError {val:"Voting on a non-existent poll".to_string()});
    }

    let mut my_poll = POLL.load(deps.storage,poll_name.clone())?;

    if choice == "yes" {
        my_poll.yes_votes +=1;
        POLL.save(deps.storage,poll_name, &my_poll)?;
        Ok(Response::new().add_attribute("action", "voted_yes"))
    } else if choice== "no" {
        my_poll.no_votes +=1;
        POLL.save(deps.storage,poll_name, &my_poll)?;
        Ok(Response::new().add_attribute("action", "voted_no"))
    } else {
        return Err(ContractError::CustomError {val: "invalid option when voting".to_string()});
    }

 //   match choice {
  //      "yes" => {my_poll.yes_votes++;},
   //     "no" =>{my_poll.no_votes +=1;},
   //     _ => Err(ContractError::CustomError {val: "invalid option when voting".to_string()});
 //   }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPoll { poll_name } => {query_get_poll(deps, env, poll_name) },
        QueryMsg::GetConfig {} => to_binary(&CONFIG.load(deps.storage)?),
    }

}

fn query_get_poll(deps: Deps, env:Env, poll_name:String) ->StdResult<Binary> {
    let poll = POLL.may_load(deps.storage, poll_name)?;
    to_binary(&GetPollResponse{poll})
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, attr, from_binary, to_binary};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ExecuteMsg, GetPollResponse, InstantiateMsg, QueryMsg};
    use crate::state::{Config, SPoll};

    #[test]
    fn test_initialize() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(&"addr1".to_string(), &[]);
        let msg = InstantiateMsg{ admin_address: "addr_admin".to_string() };

        let resp = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        assert_eq!(resp.attributes, vec![
            attr("action", "instantiate")
        ]);

        let msg = QueryMsg::GetConfig {};
        let resp = query(deps.as_ref(), env, msg).unwrap();
        let config : Config = from_binary(&resp).unwrap();
        //assert_eq!(config, Config{ admin_address: Addr::unchecked("addr_admin") });
        assert_eq!(config, Config{
            admin_address: Addr::unchecked("addr_admin"),
        })
    }

    #[test]
    fn test_create_poll() {
        let mut deps = mock_dependencies();
        let info = mock_info(&"addr1".to_string(), &[]);
        let env = mock_env();

        let msg = InstantiateMsg{ admin_address: "addr1_admin".to_string() };
        let _resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    //    !assert_eq!(resp.attributes, vec![attr("action", "instantiate")]);

        let msg = ExecuteMsg::CreatePollMsg { poll_name: "question1".to_string() };
        let resp = execute(deps.as_mut(),env.clone(), info.clone(), msg).unwrap();
        assert_eq!(resp.attributes, vec![attr("action", "poll_created")]);

        let msg = ExecuteMsg::CreatePollMsg { poll_name: "question1".to_string() };
        let _resp = execute(deps.as_mut(),env.clone(), info.clone(), msg).unwrap_err();
     //   assert_eq!(resp.attributes, vec![attr("action", "poll_created")]);
    }

    #[test]
    fn test_vote_poll_non_existent() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(&"addr1".to_string(), &[]);
        let msg = InstantiateMsg{ admin_address: "addr1_admin".to_string() };
        let _resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::CreatePollMsg { poll_name: "question1".to_string() };
        let _resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::VotePollMsg { poll_name: "question2".to_string(), choice: "yes".to_string() };
        let resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

       // assert_eq!(resp.attributes, vec![attr("action", "voted_yes")]);
    }

    // #[test]
    // fn test_vote_poll_wrong_option() {
    //     let mut deps = mock_dependencies();
    //     let env = mock_env();
    //     let info = mock_info(&"addr1".to_string(), &[]);
    //     let msg = InstantiateMsg{ admin_address: "addr1_admin".to_string() };
    //     let _resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    //
    //     let msg = ExecuteMsg::CreatePollMsg { poll_name: "question1".to_string() };
    //     let _resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    //
    //     let msg = ExecuteMsg::VotePollMsg { poll_name: "question1".to_string(), choice: "maybe".to_string() };
    //     let resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    //
    //     assert_eq!(resp.attributes, vec![attr("action", "voted_yes")]);
    // }

    #[test]
    fn test_query_poll() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(&"addr1".to_string(), &[]);

        let msg = InstantiateMsg{ admin_address: "addr1_admin".to_string() };

        let resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        //assert_eq!(resp.attributes, vec![attr("action", "instantiate")]);
         // !assert_eq!(resp.attributes, vec![attr("action", "instantiate")]);

        let msg = ExecuteMsg::CreatePollMsg { poll_name: "question1".to_string() };
        let resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(resp.attributes, vec![attr("action", "poll_created")]);

        // let msg = QueryMsg::GetPoll { poll_name: "query1".to_string() };
        // let resp = query(deps.as_ref(),env.clone(), msg).unwrap();
        // let get_poll_response : GetPollResponse = from_binary(&resp).unwrap();
        // assert_eq!(get_poll_response, GetPollResponse{ poll: Some(SPoll {
        //     poll_name: "question1".to_string(),
        //     yes_votes: 0,
        //     no_votes: 0
        // }) });

        let msg = QueryMsg::GetPoll { poll_name: "query2".to_string() };
        let resp = query(deps.as_ref(),env.clone(), msg).unwrap();
        let get_poll_response : GetPollResponse = from_binary(&resp).unwrap();
        assert_eq!(get_poll_response, GetPollResponse{ poll: None});

        let msg = QueryMsg::GetPoll { poll_name: "question1".to_string() };
        let resp = query(deps.as_ref(), env.clone(), msg).unwrap();
        let get_poll_response : GetPollResponse = from_binary(&resp).unwrap();

        assert_eq!(get_poll_response, GetPollResponse{ poll: Some(SPoll {
            poll_name: "question1".to_string(),
            yes_votes: 0,
            no_votes: 0
        }) })
    }

}
