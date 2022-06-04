use crate::*;

#[get("/")]
pub async fn root() -> String {
    format!("Hello world")
}

#[get("/list-farms")]
pub async fn list_farms() -> Json<Vec<FarmInfo>> {
    let farms_map = get_redis_farms().await;

    let farms = farms_map.values().cloned().collect();

    Json(farms)
}

#[get("/list-pools")]
pub async fn list_pools() -> Json<Vec<PoolInfo>> {
    let pools_map = get_redis_pools().await;

    let pools = pools_map.values().cloned().collect();

    Json(pools)
}

#[get("/whitelisted-tokens")]
pub async fn list_whitelisted_tokens() -> Json<Vec<FungibleTokenMetadata>> {
    let tokens_map = get_redis_tokens_metadata().await;

    if tokens_map.is_empty() {
        println!("ERR_FETCHING_TOKENS_METADATA");
        return Json(vec![]);
    }

    let tokens = tokens_map.values().cloned().collect();

    Json(tokens)
}

type Result<T, E = rocket::response::Debug<Box<dyn std::error::Error>>> = std::result::Result<T, E>;

#[get("/init-redis")]
pub async fn init_redis() -> Result<()> {
    // TODO: improve this by collecting string results, like "Ok"
    println!("Redis is starting");
    let result = redis_update_tokens_metadata(None).await.expect("Done");
    println!("Get tokens finished");
    let result = redis_update_farms().await.expect("Done");
    println!("Get farms finished");
    let result = redis_update_pools().await.expect("Done");
    println!("Get pools finished");

    Ok(())
}
