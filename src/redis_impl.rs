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

pub async fn redis_update_farms(driver: BTreeMap<String, FarmInfo>) {
    let mut conn = connect();

    println!("******* Running HASH::HSET commands *******");

    let prefix = "redis-driver";

    let _: () = redis::cmd("HSET")
        .arg(format!("{}:{}", prefix, "rust"))
        .arg(driver)
        .query(&mut conn)
        .expect("failed to execute HSET");
}

// Store all farms with Running state
pub async fn update_farms() {
    let result = get_farms().await;

    let mut farms: Vec<FarmInfo> = Vec::new();

    match result {
        Ok(x) => farms = x,
        _ => println!("Error!"),
    }

    let mut driver: BTreeMap<String, FarmInfo> = BTreeMap::new();

    for farm in farms {
        driver.insert(farm.farm_id.clone(), farm);
    }

    redis_update_farms(driver);
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
