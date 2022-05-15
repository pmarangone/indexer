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

// #[tokio::main]
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
    rocket::build().mount("/hello", routes![get_url])
}
