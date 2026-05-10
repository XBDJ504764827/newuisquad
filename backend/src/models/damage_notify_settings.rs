use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DamageNotifySettings {
    pub id: i32,
    pub server_id: i32,
    pub enabled: bool,
    pub notify_kill: bool,
    pub notify_damage: bool,
    pub message_mode: String,
    pub hit_layout: String,
    pub kill_layout: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDamageNotifyRequest {
    pub enabled: Option<bool>,
    pub notify_kill: Option<bool>,
    pub notify_damage: Option<bool>,
    pub message_mode: Option<String>,
    pub hit_layout: Option<String>,
    pub kill_layout: Option<String>,
}
