use crate::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub decimal: u8,
    pub price: String,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize, RedisJsonValue)]
pub struct FarmInfo {
    pub farm_id: String,
    pub farm_kind: String,
    pub farm_status: String,
    pub seed_id: String,
    pub reward_token: String,
    pub start_at: u64,
    pub reward_per_session: String,
    pub session_interval: u64,
    pub total_reward: String,
    pub cur_round: u64,
    pub last_round: u64,
    pub claimed_reward: String,
    pub unclaimed_reward: String,
    pub beneficiary_reward: String,
}

#[derive(Debug, Serialize, Deserialize, RedisJsonValue)]
pub struct PoolInfo {
    /// Pool kind.
    pub pool_kind: String,
    /// List of tokens in the pool.
    pub token_account_ids: Vec<String>,
    /// How much NEAR this contract has.
    pub amounts: Vec<String>,
    /// Fee charged for swap.
    pub total_fee: u32,
    /// Total number of shares.
    pub shares_total_supply: String,
    pub amp: u64,
}

#[derive(Debug, Serialize, Deserialize, RedisJsonValue)]
pub struct FungibleTokenMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    // pub reference_hash: Option<Base64VecU8>,
    pub decimals: u8,
}
