use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RconLog {
    pub id: i32,
    pub server_id: i32,
    pub admin_user: String,
    pub command: String,
    pub response: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteRconRequest {
    pub command: String,
    pub admin_user: String,
}
