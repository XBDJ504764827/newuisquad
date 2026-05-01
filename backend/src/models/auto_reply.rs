use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AutoReply {
    pub id: i32, pub server_id: i32, pub keyword: String,
    pub reply_message: String, pub enabled: bool, pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAutoReply { pub keyword: String, pub reply_message: String }

#[derive(Debug, Deserialize)]
pub struct UpdateAutoReply { pub keyword: Option<String>, pub reply_message: Option<String>, pub enabled: Option<bool> }
