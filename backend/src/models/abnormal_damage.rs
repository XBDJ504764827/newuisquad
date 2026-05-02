use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AbnormalDamageConfig {
    pub id: i32,
    pub server_id: i32,
    pub enabled: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAbnormalDamageConfigRequest {
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AbnormalDamageRule {
    pub id: i32,
    pub server_id: i32,
    pub max_damage: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAbnormalDamageRule {
    pub max_damage: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AbnormalDamageLog {
    pub id: i32,
    pub server_id: i32,
    pub player_name: String,
    pub player_steamid64: String,
    pub victim_name: String,
    pub victim_steamid64: String,
    pub weapon: String,
    pub damage: i32,
    pub attacker_faction: String,
    pub victim_faction: String,
    pub logged_at: DateTime<Utc>,
}
