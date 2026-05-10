use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TkSettings {
    pub id: i32,
    pub server_id: i32,
    pub enabled: bool,
    pub max_team_kills: i32,
    pub apology_time_minutes: i32,
    pub apology_keyword: String,
    pub notification_message: Option<String>,
    pub tk_broadcast_message: Option<String>,
    pub apology_pre_window_secs: i32,
    pub tk_attacker_msg: String,
    pub tk_victim_msg: String,
    pub tk_broadcast_msg: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTkSettingsRequest {
    pub enabled: Option<bool>,
    pub max_team_kills: Option<i32>,
    pub apology_time_minutes: Option<i32>,
    pub apology_keyword: Option<String>,
    pub notification_message: Option<String>,
    pub tk_broadcast_message: Option<String>,
    pub apology_pre_window_secs: Option<i32>,
    pub tk_attacker_msg: Option<String>,
    pub tk_victim_msg: Option<String>,
    pub tk_broadcast_msg: Option<String>,
}
