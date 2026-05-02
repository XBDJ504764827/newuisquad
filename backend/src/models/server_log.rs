use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServerLog {
    pub id: i32,
    pub server_id: i32,
    pub log_level: String,
    pub category: Option<String>,
    pub message: String,
    pub raw_line: Option<String>,
    pub logged_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct LogPage {
    pub data: Vec<ServerLog>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct LogEntry {
    pub server_id: i32,
    pub log_level: String,
    pub category: Option<String>,
    pub message: String,
    pub raw_line: Option<String>,
    pub logged_at: DateTime<Utc>,
}
