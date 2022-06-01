use crate::*;

// #[get("/list-seeds")]
// pub async fn list_seeds() -> Json<HashMap<String, String>> {
//     let val = get_seeds().await;

//     let mut value: HashMap<String, String> = HashMap::new();

//     match val {
//         Ok(x) => value = x,
//         _ => println!("Error!"),
//     }

//     Json(value)
// }

#[get("/list-farms")]
pub async fn list_farms() -> Json<Vec<FarmInfo>> {
    let res = get_redis_farms().await;
    let mut farms: Vec<FarmInfo> = Vec::new();

    for (_, farm) in res.0 {
        farms.push(farm);
    }

    Json(farms)
}

#[get("/list-pools")]
pub async fn list_pools() -> Json<Vec<PoolInfo>> {
    let res = get_redis_pools().await;
    let mut pools: Vec<PoolInfo> = Vec::new();

    for (_, pool) in res.0 {
        pools.push(pool);
    }

    Json(pools)
}

#[get("/whitelisted-tokens")]
pub async fn list_whitelisted_tokens() -> Json<Vec<String>> {
    let result = get_whitelisted_tokens().await;

    let mut tokens: Vec<String> = Vec::new();

    match result {
        Ok(x) => tokens = x,
        _ => println!("Error"),
    }

    Json(tokens)
}

type Result<T, E = rocket::response::Debug<Box<dyn std::error::Error>>> = std::result::Result<T, E>;

// TODO: the following method should call every function that create/update redis state
#[get("/init-redis")]
pub async fn init_redis() -> Result<()> {
    // TODO: improve this by collecting string results, like "Ok" and return Vec<String>
    println!("Redis is starting");
    let result = redis_update_farms().await.expect("Hello world");
    println!("Get farms finished");
    let result = redis_update_pools().await.expect("Hello there");
    println!("Get pools finished");

    Ok(())
}
