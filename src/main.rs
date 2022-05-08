use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    decimal: u8,
    price: String,
    symbol: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let example: String = String::from("https://httpbin.org/ip");
    let path: String = String::from("https://testnet-indexer.ref-finance.com/list-token-price");

    let resp: HashMap<String, TokenInfo> = reqwest::get(path).await?.json().await?;
    println!("{:#?}", resp);

    Ok(())
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let example: String = String::from("https://httpbin.org/ip");
//     let path: String = String::from("https://testnet-indexer.ref-finance.com/list-token-price");

//     let body = reqwest::get(path).await?.text().await?;
//     println!("body = {:?}", body);
//     Ok(())
// }
