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

mod consts;
mod models;
mod redis_impl;
mod routes;
mod utils;

use consts::*;
use models::*;
use redis_impl::*;
use utils::*;

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

    if let QueryResponseKind::CallResult(result) = response.kind {
        seeds = from_slice::<HashMap<String, String>>(&result.result)?;
    }

    Ok(seeds)
}

async fn get_farms() -> Result<Vec<FarmInfo>, Box<dyn std::error::Error>> {
    let seeds = get_seeds().await.unwrap_or_default();

    if seeds.is_empty() {
        println!("ERR_FETCHING_SEEDS");
        return Ok(vec![]);
    }

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
            farms.extend(res);
        }
    }

    if farms.is_empty() {
        println!("ERR_FETCHING_FARMS");
    }

    Ok(farms)
}

async fn get_pools() -> Result<Vec<PoolInfo>, Box<dyn std::error::Error>> {
    let mut pools: Vec<PoolInfo> = Vec::new();
    let token_metadata: BTreeMap<String, FungibleTokenMetadata> = get_redis_tokens_metadata().await;
    let seeds = internal_farm_seeds().await.unwrap_or_default();

    if seeds.is_empty() {
        println!("ERR_FETCHING_SEEDS");
    }

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
            } else {
                let vec = &vec![token.clone()];
                let metadata = internal_update_token_metadata(vec)
                    .await
                    .unwrap_or_default();

                if !metadata.is_empty() {
                    let symbol = metadata.get(token).unwrap().symbol.clone();
                    symbols.push(symbol);

                    redis_update_tokens_metadata(Some(metadata));
                }
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
    tokens: &Vec<String>,
) -> Result<BTreeMap<String, FungibleTokenMetadata>, Box<dyn std::error::Error>> {
    let mut valid_tokens: BTreeMap<String, FungibleTokenMetadata> = BTreeMap::new();

    for token_contract in tokens.iter() {
        let args = FunctionArgs::from(json!({}).to_string().into_bytes());
        let response = call_view(token_contract, Methods::FtMetadata.value(), args).await?;

        if let QueryResponseKind::CallResult(result) = response.kind {
            let metadata = from_slice::<FungibleTokenMetadata>(&result.result)?;
            valid_tokens.insert(token_contract.clone(), metadata);
        }
    }

    Ok(valid_tokens)
}

pub async fn get_whitelisted_tokens(
) -> Result<BTreeMap<String, FungibleTokenMetadata>, Box<dyn std::error::Error>> {
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

    let valid_tokens = internal_update_token_metadata(&tokens)
        .await
        .unwrap_or_default();

    Ok(valid_tokens)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![
            routes::root,
            routes::init_redis,
            routes::list_farms,
            routes::list_pools,
            routes::list_whitelisted_tokens,
        ],
    )
}
