use crate::*;

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

#[get("/list-pools")]
pub async fn list_pools() -> Json<Vec<PoolInfo>> {
    let result = get_pools().await;

    let mut pools: Vec<PoolInfo> = Vec::new();

    match result {
        Ok(x) => pools = x,
        _ => println!("Error"),
    }

    Json(pools)
}

#[get("/whitelisted-tokens")]
pub async fn list_whitelisted_tokens() -> Json<Vec<String>> {
    let result = whitelisted_tokens().await;

    let mut tokens: Vec<String> = Vec::new();

    match result {
        Ok(x) => tokens = x,
        _ => println!("Error"),
    }

    Json(tokens)
}

// TODO: the following method should call every function that create/update redis state
// #[get("/init-redis")]
// pub async fn init_redis() -> Json<String> {
//     // TODO: improve this by collecting string results, like "Ok" and return Vec<String>
//     let result = get_farms().await;

//     Json("".to_string())
// }
