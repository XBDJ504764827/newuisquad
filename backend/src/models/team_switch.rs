use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamSwitchConfig {
    pub id: i32,
    pub server_id: i32,
    pub enabled: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamSwitchConfigRequest {
    pub enabled: Option<bool>,
}
