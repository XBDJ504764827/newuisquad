use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DamageNotifySettings {
    pub id: i32,
    pub server_id: i32,
    pub enabled: bool,
    pub keyword: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDamageNotifyRequest {
    pub enabled: Option<bool>,
    pub keyword: Option<String>,
}
