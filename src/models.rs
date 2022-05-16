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
