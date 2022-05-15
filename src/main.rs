#[macro_use]
extern crate rocket;

use std::collections::HashMap;

use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::QueryRequest;

use rocket::serde::json::{json, Json};
use serde::{Deserialize, Serialize};
use serde_json::from_slice;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    decimal: u8,
    price: String,
    symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FarmInfo {
    farm_id: String,
    farm_kind: String,
    farm_status: String,
    seed_id: String,
    reward_token: String,
    start_at: u64,
    reward_per_session: String,
    session_interval: u64,
    total_reward: String,
    cur_round: u64,
    last_round: u64,
    claimed_reward: String,
    unclaimed_reward: String,
    beneficiary_reward: String,
}

async fn contract_version() -> Result<String, Box<dyn std::error::Error>> {
    let client = JsonRpcClient::connect("https://rpc.testnet.near.org");

    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: "auto-compounder-001.fluxusfi.testnet".parse()?,
            method_name: "contract_version".to_string(),
            args: FunctionArgs::from(json!({}).to_string().into_bytes()),
        },
    };

    let response = client.call(request).await?;

    let mut res: String = String::from("");

    /* What is response, and how to assign to variable only if query was successful */
    if let QueryResponseKind::CallResult(result) = response.kind {
        res = from_slice::<String>(&result.result)?;
        println!("{:#?}", from_slice::<String>(&result.result)?);
    }

    Ok(res)
}

#[get("/contract-version")]
pub async fn get_contract_version() -> String {
    let res = contract_version().await;

    let mut value: String = String::from("Failed");

    match res {
        Ok(x) => value = x,
        _ => println!("Error"),
    }

    value
}

async fn get_seeds() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let client = JsonRpcClient::connect("https://rpc.testnet.near.org");

    let contract: &str = "v2.ref-farming.testnet";
    let method_name: String = String::from("list_seeds");

    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: contract.parse()?,
            method_name: method_name,
            args: FunctionArgs::from(
                json!({
                    "from_index": 0u64,
                    "limit": 100u64
                })
                .to_string()
                .into_bytes(),
            ),
        },
    };

    let response = client.call(request).await?;

    let mut res: HashMap<String, String> = HashMap::new();

    /* What is response, and how to assign to variable only if query was successful */
    if let QueryResponseKind::CallResult(result) = response.kind {
        res = from_slice::<HashMap<String, String>>(&result.result)?;
    }

    Ok(res)
}

#[get("/list-seeds")]
pub async fn list_seeds() -> Json<HashMap<String, String>> {
    let val = get_seeds().await;

    let mut value: HashMap<String, String> = HashMap::new();

    match val {
        Ok(x) => value = x,
        _ => println!("Error!"),
    }

    Json(value)
}

async fn get_farms() -> Result<Vec<FarmInfo>, Box<dyn std::error::Error>> {
    let client = JsonRpcClient::connect("https://rpc.testnet.near.org");

    let contract: &str = "v2.ref-farming.testnet";
    let method_name: String = String::from("list_seeds");

    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: contract.parse()?,
            method_name: method_name,
            args: FunctionArgs::from(
                json!({
                    "from_index": 0u64,
                    "limit": 100u64
                })
                .to_string()
                .into_bytes(),
            ),
        },
    };

    let response = client.call(request).await?;

    let mut seeds: HashMap<String, String> = HashMap::new();

    if let QueryResponseKind::CallResult(result) = response.kind {
        seeds = from_slice::<HashMap<String, String>>(&result.result)?;
    }

    assert!(seeds.len() > 0, "ERR_FETCHING_SEEDS");

    let method_name: String = String::from("list_farms_by_seed");

    let mut farms: Vec<FarmInfo> = Vec::new();

    for (key, _) in &seeds {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
                account_id: contract.parse()?,
                method_name: method_name.clone(),
                args: FunctionArgs::from(
                    json!({
                        "seed_id": key,
                    })
                    .to_string()
                    .into_bytes(),
                ),
            },
        };

        let response = client.call(request).await?;

        /* What is response, and how to assign to variable only if query was successful */
        if let QueryResponseKind::CallResult(result) = response.kind {
            let res: Vec<FarmInfo> = from_slice::<Vec<FarmInfo>>(&result.result)?;
            for farm in res {
                let status = farm.farm_status.clone();
                let running: String = String::from("Running");

                if status == running {
                    farms.push(farm);
                }
            }
        }
    }

    assert!(farms.len() > 0, "ERR_FETCHING_FARMS");

    Ok(farms)
}

#[get("/list-farms")]
pub async fn list_farms() -> Json<Vec<FarmInfo>> {
    let result = get_farms().await;

    let mut farms: Vec<FarmInfo> = Vec::new();

    match result {
        Ok(x) => farms = x,
        _ => println!("Error!"),
    }

    Json(farms)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![get_contract_version, list_seeds, list_farms])
}
