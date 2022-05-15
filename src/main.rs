#[macro_use]
extern crate rocket;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    decimal: u8,
    price: String,
    symbol: String,
}

use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::QueryRequest;

use env_logger;
use serde_json::{from_slice, json};


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

async fn get_response() -> Result<String, Box<dyn std::error::Error>> {
    let path: String = String::from("https://testnet-indexer.ref-finance.com/list-token-price");

    let resp: HashMap<String, TokenInfo> = reqwest::get(path).await?.json().await?;
    println!("{:#?}", resp);

    let mut res: String = String::from("Failed");

    for (key, value) in resp.iter() {
        res = value.symbol.clone();
        break;
    }

    println!("The sample will be {}", res);

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

#[get("/url")]
pub async fn get_url() -> String {
    let val = get_response().await;

    let mut value: String = String::from("Failed");

    match val {
        Ok(x) => value = x,
        _ => println!("Error!"),
    }

    println!("{}", value);

    String::from("")
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/hello", routes![get_url, get_contract_version])
}
