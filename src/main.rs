#[macro_use]
extern crate rocket;

use std::collections::BTreeMap;
use std::collections::HashMap;

use near_jsonrpc_client::methods::query::RpcQueryResponse;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::QueryRequest;

use derive_redis_json::RedisJsonValue;
use redis::Commands;
use rocket::serde::json::{json, Json};
use serde::{Deserialize, Serialize};
use serde_json::from_slice;

mod models;
mod routes;

use models::*;

mod redis_impl;
use redis_impl::*;

const exchange_id: &str = "ref-finance-101.testnet";
const farm_id: &str = "v2.ref-farming.testnet";

// https://stackoverflow.com/a/36928678
enum Methods {
    NumPools,
    GetPools,
    ListSeeds,
    ListFarmsBySeeds,
}

impl Methods {
    fn value(&self) -> String {
        match *self {
            Methods::NumPools => String::from("get_number_of_pools"),
            Methods::GetPools => String::from("get_pools"),
            Methods::ListSeeds => String::from("list_seeds"),
            Methods::ListFarmsBySeeds => String::from("list_farms_by_seed"),
        }
    }
}

async fn contract_version() -> Result<String, Box<dyn std::error::Error>> {
    let contract = "auto-compounder-001.fluxusfi.testnet";
    let method_name = "contract_version".to_string();
    let args = FunctionArgs::from(json!({}).to_string().into_bytes());

    let response = call_view(contract, method_name, args).await?;

    let mut res: String = String::from("");

    /* What is response, and how to assign to variable only if query was successful */
    if let QueryResponseKind::CallResult(result) = response.kind {
        res = from_slice::<String>(&result.result)?;
        println!("{:#?}", from_slice::<String>(&result.result)?);
    }

    Ok(res)
}

async fn get_seeds() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let args = FunctionArgs::from(
        json!({
            "from_index": 0u64,
            "limit": 100u64
        })
        .to_string()
        .into_bytes(),
    );

    let response = call_view(farm_id, Methods::ListSeeds.value(), args).await?;

    let mut res: HashMap<String, String> = HashMap::new();

    /* What is response, and how to assign to variable only if query was successful */
    if let QueryResponseKind::CallResult(result) = response.kind {
        res = from_slice::<HashMap<String, String>>(&result.result)?;
    }

    Ok(res)
}

async fn get_farms() -> Result<Vec<FarmInfo>, Box<dyn std::error::Error>> {
    // TODO: get number of seeds, then get all seeds
    let args = FunctionArgs::from(
        json!({
            "from_index": 0u64,
            "limit": 100u64
        })
        .to_string()
        .into_bytes(),
    );

    let response = call_view(farm_id, Methods::ListSeeds.value(), args).await?;

    let mut seeds: HashMap<String, String> = HashMap::new();

    if let QueryResponseKind::CallResult(result) = response.kind {
        seeds = from_slice::<HashMap<String, String>>(&result.result)?;
    }

    // TODO: improve error handling
    assert!(seeds.len() > 0, "ERR_FETCHING_SEEDS");

    let mut farms: Vec<FarmInfo> = Vec::new();

    for (key, _) in &seeds {
        let args = FunctionArgs::from(
            json!({
                "seed_id": key,
            })
            .to_string()
            .into_bytes(),
        );

        let response = call_view(farm_id, Methods::ListFarmsBySeeds.value(), args).await?;

        if let QueryResponseKind::CallResult(result) = response.kind {
            let res: Vec<FarmInfo> = from_slice::<Vec<FarmInfo>>(&result.result)?;
            // TODO: refactor in preference of collect
            for farm in res {
                farms.push(farm);
            }
        }
    }

    // TODO: improve error handling
    assert!(farms.len() > 0, "ERR_FETCHING_FARMS");

    Ok(farms)
}

async fn call_view(
    contract: &str,
    method_name: String,
    args: FunctionArgs,
) -> Result<RpcQueryResponse, Box<dyn std::error::Error>> {
    let client = JsonRpcClient::connect("https://rpc.testnet.near.org");

    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: contract.parse()?,
            method_name: method_name,
            args: args,
        },
    };

    let response = client.call(request).await?;
    Ok(response)
}

async fn get_pools() -> Result<Vec<PoolInfo>, Box<dyn std::error::Error>> {
    let mut pools: Vec<PoolInfo> = Vec::new();
    // let token_metadata: HashMap<String, FungibleTokenMetadata> = HashMap::new();
    // let seeds: Vec<String> = Vec::new();

    let method_name = Methods::NumPools.value();

    let args = FunctionArgs::from(json!({}).to_string().into_bytes());
    let response = call_view(exchange_id, method_name, args).await?;

    let mut num_pools: u64 = 0;
    if let QueryResponseKind::CallResult(result) = response.kind {
        num_pools = from_slice::<u64>(&result.result)?;
    }

    let mut base_index = 0;
    while base_index < num_pools {
        let args = FunctionArgs::from(
            json!({
                "from_index": base_index,
                "limit": 200u64
            })
            .to_string()
            .into_bytes(),
        );

        let response = call_view(exchange_id, Methods::GetPools.value(), args).await?;

        if let QueryResponseKind::CallResult(result) = response.kind {
            let mut batch_pools = from_slice::<Vec<PoolInfo>>(&result.result)?;
            base_index += batch_pools.len() as u64;

            pools.append(&mut batch_pools);
        }
    }
    /*
        query whitelisted_tokens
        update token metadata on redis
        impl get_token_metadata

    */

    // for pool in pools.iter_mut() {
    //     if true {
    //         pool.farming = Some(true);
    //     }
    // }

    Ok(pools)
}

pub async fn internal_farm_seeds() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let farms = get_redis_farms().await;
    let mut seeds: Vec<String> = Vec::new();

    for (_, farm) in farms.iter() {
        let status = &farm.farm_status;
        let total_reward: u128 = farm.total_reward.parse::<u128>().unwrap();
        let claimed_reward: u128 = farm.claimed_reward.parse::<u128>().unwrap();
        let unclaimed_reward: u128 = farm.unclaimed_reward.parse::<u128>().unwrap();

        if *status == "Running".to_string() && total_reward > claimed_reward + unclaimed_reward {
            seeds.push(farm.seed_id.clone());
        }
    }

    Ok(seeds)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![
            routes::get_contract_version,
            routes::init_redis,
            routes::list_seeds,
            routes::list_farms,
            routes::list_pools,
        ],
    )
}
