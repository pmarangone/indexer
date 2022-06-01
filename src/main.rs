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
use rocket::serde::json::{json, Json};
use serde::{Deserialize, Serialize};
use serde_json::from_slice;

mod models;
mod routes;

use models::*;

mod redis_impl;
use redis_impl::*;

enum Contracts {
    RefExchange,
    RefFarm,
    FluxusFarm,
    FluxusSafe,
}

impl Contracts {
    fn value(&self) -> &str {
        match *self {
            Contracts::RefExchange => "ref-finance-101.testnet",
            Contracts::RefFarm => "v2.ref-farming.testnet",
            Contracts::FluxusFarm => "farm101.fluxusfi.testnet",
            Contracts::FluxusSafe => "safe-004.fluxusfi.testnet",
        }
    }
}

// https://stackoverflow.com/a/36928678
enum Methods {
    NumPools,
    GetPools,
    ListSeeds,
    ListFarmsBySeeds,
    WhitelistedTokens,
    FtMetadata,
}

impl Methods {
    fn value(&self) -> String {
        match *self {
            Methods::NumPools => String::from("get_number_of_pools"),
            Methods::GetPools => String::from("get_pools"),
            Methods::ListSeeds => String::from("list_seeds"),
            Methods::ListFarmsBySeeds => String::from("list_farms_by_seed"),
            Methods::WhitelistedTokens => String::from("get_whitelisted_tokens"),
            Methods::FtMetadata => String::from("ft_metadata"),
        }
    }
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

    let response = call_view(Contracts::RefFarm.value(), Methods::ListSeeds.value(), args).await?;

    let mut seeds: HashMap<String, String> = HashMap::new();

    /* What is response, and how to assign to variable only if query was successful */
    if let QueryResponseKind::CallResult(result) = response.kind {
        seeds = from_slice::<HashMap<String, String>>(&result.result)?;
    }

    Ok(seeds)
}

async fn get_farms() -> Result<Vec<FarmInfo>, Box<dyn std::error::Error>> {
    let placeholder: HashMap<String, String> = HashMap::new();

    // TODO: handle this better
    let seeds = get_seeds().await.unwrap_or_else(|err| placeholder);

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

        let response = call_view(
            Contracts::RefFarm.value(),
            Methods::ListFarmsBySeeds.value(),
            args,
        )
        .await?;

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
    let token_metadata: Json<BTreeMap<String, FungibleTokenMetadata>> =
        get_redis_token_metadata().await;
    let seeds = internal_farm_seeds().await?;

    assert!(seeds.len() > 0, "ERR");

    let method_name = Methods::NumPools.value();

    let args = FunctionArgs::from(json!({}).to_string().into_bytes());
    let response = call_view(Contracts::RefExchange.value(), method_name, args).await?;

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

        let response = call_view(
            Contracts::RefExchange.value(),
            Methods::GetPools.value(),
            args,
        )
        .await?;

        if let QueryResponseKind::CallResult(result) = response.kind {
            let mut batch_pools = from_slice::<Vec<PoolInfo>>(&result.result)?;
            base_index += batch_pools.len() as u64;

            pools.append(&mut batch_pools);
        }
    }

    for (idx, pool) in pools.iter_mut().enumerate() {
        pool.id = Some(idx as u64);

        let lpt_id: String = format!("{}@{}", Contracts::RefExchange.value(), idx);
        if seeds.contains(&lpt_id) {
            pool.farming = Some(true);
        } else {
            pool.farming = Some(false);
        }

        let mut symbols: Vec<String> = Vec::new();

        for token in pool.token_account_ids.iter() {
            if token_metadata.contains_key(token) {
                let metadata = token_metadata.get(token).unwrap();
                let symbol: String = metadata.symbol.clone();
                symbols.push(symbol);
            }
        }

        let _ = pool.token_symbols.insert(symbols);
    }

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

pub async fn internal_update_token_metadata(
    contract: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut is_valid = false;

    let args = FunctionArgs::from(json!({}).to_string().into_bytes());
    let response = call_view(contract, Methods::FtMetadata.value(), args).await?;

    let mut metadata: FungibleTokenMetadata = FungibleTokenMetadata {
        spec: "".to_string(),
        name: "".to_string(),
        symbol: "".to_string(),
        icon: Some("".to_string()),
        reference: Some("".to_string()),
        reference_hash: Some("".to_string()),
        decimals: 0u8,
    };

    if let QueryResponseKind::CallResult(result) = response.kind {
        metadata = from_slice::<FungibleTokenMetadata>(&result.result)?;
        is_valid = true;
    }

    if is_valid {
        redis_add_token_metadata(contract, metadata).await;
    }

    Ok(())
}

pub async fn get_whitelisted_tokens() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let args = FunctionArgs::from(json!({}).to_string().into_bytes());
    let response = call_view(
        Contracts::RefExchange.value(),
        Methods::WhitelistedTokens.value(),
        args,
    )
    .await?;

    let mut tokens: Vec<String> = Vec::new();

    if let QueryResponseKind::CallResult(result) = response.kind {
        tokens = from_slice::<Vec<String>>(&result.result)?;
    }

    for token in tokens.iter() {
        internal_update_token_metadata(token).await;
    }

    Ok(tokens)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![
            routes::init_redis,
            routes::list_farms,
            routes::list_pools,
            routes::list_whitelisted_tokens,
        ],
    )
}
