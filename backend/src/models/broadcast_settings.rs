use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BroadcastSettings {
    pub id: i32,
    pub server_id: i32,
    pub join_message_enabled: bool,
    pub join_message: String,
    pub gameop_list_enabled: bool,
    pub gameop_list_message: String,
    pub announcement_enabled: bool,
    pub announcement_content: Option<String>,
    pub announcement_interval: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBroadcastRequest {
    pub join_message_enabled: Option<bool>,
    pub join_message: Option<String>,
    pub gameop_list_enabled: Option<bool>,
    pub gameop_list_message: Option<String>,
    pub announcement_enabled: Option<bool>,
    pub announcement_content: Option<String>,
    pub announcement_interval: Option<i32>,
}
