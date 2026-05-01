use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Announcement {
    pub id: i32, pub server_id: i32, pub content: String,
    pub interval_minutes: i32, pub enabled: bool, pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnnouncement { pub content: String, pub interval_minutes: i32 }

#[derive(Debug, Deserialize)]
pub struct UpdateAnnouncement { pub content: Option<String>, pub interval_minutes: Option<i32>, pub enabled: Option<bool> }
