#[macro_use]
extern crate rocket;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::env;

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

use models::FarmInfo;

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
        println!("{:#?}", seeds);
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

fn connect() -> redis::Connection {
    //format - host:port
    let redis_host_name =
        env::var("REDIS_HOSTNAME").expect("missing environment variable REDIS_HOSTNAME");
    let redis_password = env::var("REDIS_PASSWORD").unwrap_or_default();

    //if Redis server needs secure connection
    let uri_scheme = match env::var("IS_TLS") {
        Ok(_) => "rediss",
        Err(_) => "redis",
    };

    let redis_conn_url = format!("{}://:{}@{}", uri_scheme, redis_password, redis_host_name);
    println!("{}", redis_conn_url);

    redis::Client::open(redis_conn_url)
        .expect("Invalid connection URL")
        .get_connection()
        .expect("failed to connect to Redis")
}

pub fn redis_update_farms(driver: BTreeMap<String, FarmInfo>) {
    let mut conn = connect();

    println!("******* Running HASH::HSET commands *******");

    let prefix = "redis-driver";

    let _: () = redis::cmd("HSET")
        .arg(format!("{}:{}", prefix, "rust"))
        .arg(driver)
        .query(&mut conn)
        .expect("failed to execute HSET");
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![
            routes::get_contract_version,
            routes::list_seeds,
            routes::list_farms,
            routes::update_farms,
            routes::get_redis_farms_
        ],
    )
}
