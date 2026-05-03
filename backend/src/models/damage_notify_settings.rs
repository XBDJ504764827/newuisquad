use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DamageNotifySettings {
    pub id: i32,
    pub server_id: i32,
    pub enabled: bool,
    pub keyword: String,
    pub min_damage: f64,
    pub notify_tk: bool,
    pub notify_damage: bool,
    pub notify_high_damage: bool,
    pub high_damage_threshold: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDamageNotifyRequest {
    pub enabled: Option<bool>,
    pub keyword: Option<String>,
    pub min_damage: Option<f64>,
    pub notify_tk: Option<bool>,
    pub notify_damage: Option<bool>,
    pub notify_high_damage: Option<bool>,
    pub high_damage_threshold: Option<f64>,
}
