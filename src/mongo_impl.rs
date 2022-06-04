use crate::*;

// This trait is required to use `try_next()` on the cursor
use futures::stream::TryStreamExt;

use mongodb::{options::ClientOptions, Client};

pub async fn mongo_connection() -> Result<Client, Box<dyn std::error::Error>> {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;

    // Manually set an option.
    client_options.app_name = Some("My App".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;

    Ok(client)
}

pub async fn mongo_add_tokens() -> Result<String, Box<dyn std::error::Error>> {
    let client = mongo_connection().await.unwrap();

    // Get a handle to a database.
    let db = client.database("db");

    // Get a handle to a collection
    let typed_collection = db.collection::<FungibleTokenMetadata>("ft_metadata");

    let tokens_metadata = get_whitelisted_tokens().await.unwrap_or_default();

    let mut tokens: Vec<FungibleTokenMetadata> = Vec::new();

    for (_, metadata) in tokens_metadata {
        tokens.push(metadata);
    }

    // Insert
    typed_collection.insert_many(tokens, None).await?;

    Ok(format!("Done"))
}

pub async fn mongo_add_farms() -> Result<String, Box<dyn std::error::Error>> {
    let client = mongo_connection().await.unwrap();

    // Get a handle to a database.
    let db = client.database("db");

    // Get a handle to a collection
    let typed_collection = db.collection::<FarmInfo>("farms");

    let farms = get_farms().await.unwrap_or_default();

    // Insert
    typed_collection.insert_many(farms, None).await?;

    Ok(format!("Done"))
}
pub async fn mongo_add_pools() -> Result<String, Box<dyn std::error::Error>> {
    let client = mongo_connection().await.unwrap();

    // Get a handle to a database.
    let db = client.database("db");

    // Get a handle to a collection
    let typed_collection = db.collection::<PoolInfo>("pools");

    let pools = get_pools().await.unwrap_or_default();

    // Insert
    typed_collection.insert_many(pools, None).await?;
    Ok(format!("Done"))
}

pub async fn mongo_get_tokens_metadata(
) -> Result<Vec<FungibleTokenMetadata>, Box<dyn std::error::Error>> {
    let client = mongo_connection().await.unwrap();

    // Get a handle to a database.
    let db = client.database("db");

    // Get a handle to a collection
    let typed_collection = db.collection::<FungibleTokenMetadata>("ft_metadata");

    // Query
    let mut cursor = typed_collection.find(None, None).await?;

    let mut tokens: Vec<FungibleTokenMetadata> = Vec::new();

    // Iterate over the results of the cursor.
    while let Some(token) = cursor.try_next().await? {
        println!("token name: {}", token.name);
        tokens.push(token);
    }

    Ok(tokens)
}

pub async fn mongo_get_farms() -> Result<Vec<FarmInfo>, Box<dyn std::error::Error>> {
    let client = mongo_connection().await.unwrap();

    // Get a handle to a database.
    let db = client.database("db");

    // Get a handle to a collection
    let typed_collection = db.collection::<FarmInfo>("farms");

    // Query
    let mut cursor = typed_collection.find(None, None).await?;

    let mut farms: Vec<FarmInfo> = Vec::new();

    // Iterate over the results of the cursor.
    while let Some(farm) = cursor.try_next().await? {
        println!("token name: {:#?}", farm.seed_id);
        farms.push(farm);
    }

    Ok(farms)
}
pub async fn mongo_get_pools() -> Result<Vec<PoolInfo>, Box<dyn std::error::Error>> {
    let client = mongo_connection().await.unwrap();

    // Get a handle to a database.
    let db = client.database("db");

    // Get a handle to a collection
    let typed_collection = db.collection::<PoolInfo>("pools");

    // Query
    let mut cursor = typed_collection.find(None, None).await?;

    let mut pools: Vec<PoolInfo> = Vec::new();

    // Iterate over the results of the cursor.
    while let Some(pool) = cursor.try_next().await? {
        println!("token name: {:#?}", pool.token_account_ids);
        pools.push(pool);
    }

    Ok(pools)
}
