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

// TODO: the following method should call every function that create/update redis state
#[get("/init-redis")]
pub async fn init_redis() {
    update_farms().await;
}
