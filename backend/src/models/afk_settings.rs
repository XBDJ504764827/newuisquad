use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AfkSettings {
    pub id: i32,
    pub server_id: i32,
    pub enabled: bool,
    pub min_players_to_check: i32,
    pub max_afk_minutes: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAfkSettingsRequest {
    pub enabled: Option<bool>,
    pub min_players_to_check: Option<i32>,
    pub max_afk_minutes: Option<i32>,
}
