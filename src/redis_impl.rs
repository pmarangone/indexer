use crate::*;

use std::env;

/***************************
    Redis implementation
****************************/

// Connect to redis
pub fn connect() -> redis::Connection {
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

pub async fn redis_update_farms() -> Result<String, Box<dyn std::error::Error>> {
    let mut conn = connect();

    let res = get_farms().await;
    let farms = res.unwrap();

    let mut driver: BTreeMap<String, FarmInfo> = BTreeMap::new();

    for farm in farms.clone() {
        driver.insert(farm.farm_id.clone(), farm);
    }

    println!("******* Running HASH::HSET commands *******");

    let prefix = "redis-driver";

    let _: () = redis::cmd("HSET")
        .arg(format!("{}:{}", prefix, "rust"))
        .arg(driver)
        .query(&mut conn)
        .expect("failed to execute HSET");

    Ok(format!("Hello world"))
}

pub async fn redis_update_pools() -> Result<String, Box<dyn std::error::Error>> {
    let mut conn = connect();

    let res = get_pools().await;
    let pools = res.unwrap();

    let mut driver: BTreeMap<String, PoolInfo> = BTreeMap::new();

    for pool in pools.clone() {
        driver.insert(pool.id.unwrap().to_string(), pool);
    }

    println!("******* Running HASH::HSET commands *******");

    let prefix = "redis-driver";

    let _: () = redis::cmd("HSET")
        .arg(format!("{}:{}", prefix, "pool"))
        .arg(driver)
        .query(&mut conn)
        .expect("failed to execute HSET");

    Ok(format!("Hello there"))
}

pub async fn redis_update_tokens_metadata() -> Result<String, Box<dyn std::error::Error>> {
    let mut conn = connect();

    let res = get_whitelisted_tokens().await;
    let tokens_metadata = res.unwrap();

    println!("******* Running HASH::HSET commands *******");

    let prefix = "redis-driver";

    let _: () = redis::cmd("HSET")
        .arg(format!("{}:{}", prefix, "metadata"))
        .arg(tokens_metadata)
        .query(&mut conn)
        .expect("failed to execute HSET");

    Ok(format!("Yoo"))
}

pub async fn get_redis_tokens_metadata() -> Json<BTreeMap<String, FungibleTokenMetadata>> {
    let mut conn = connect();

    println!("******* Running HASH::HGETALL commands *******");

    let prefix = "redis-driver";

    let info: BTreeMap<String, FungibleTokenMetadata> = redis::cmd("HGETALL")
        .arg(format!("{}:{}", prefix, "metadata"))
        .query(&mut conn)
        .expect("failed to execute HGETALL");

    Json(info)
}

pub async fn get_redis_farms() -> Json<BTreeMap<String, FarmInfo>> {
    let mut conn = connect();

    println!("******* Running HASH::HGETALL commands *******");

    let prefix = "redis-driver";

    let info: BTreeMap<String, FarmInfo> = redis::cmd("HGETALL")
        .arg(format!("{}:{}", prefix, "rust"))
        .query(&mut conn)
        .expect("failed to execute HGETALL");

    Json(info)
}

pub async fn get_redis_pools() -> Json<BTreeMap<String, PoolInfo>> {
    let mut conn = connect();

    println!("******* Running HASH::HGETALL commands *******");

    let prefix = "redis-driver";

    let info: BTreeMap<String, PoolInfo> = redis::cmd("HGETALL")
        .arg(format!("{}:{}", prefix, "pool"))
        .query(&mut conn)
        .expect("failed to execute HGETALL");

    Json(info)
}
