use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminUser {
    pub id: i32,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub permissions: JsonValue,
    pub steam_id64: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAdminRequest {
    pub username: String,
    pub password: String,
    #[serde(default = "default_role")]
    pub role: String,
    pub permissions: Option<JsonValue>,
    pub steam_id64: Option<String>,
    pub notes: Option<String>,
}

fn default_role() -> String { "巡查员".to_string() }

#[derive(Debug, Deserialize)]
pub struct UpdateAdminRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
    pub permissions: Option<JsonValue>,
    pub steam_id64: Option<String>,
    pub notes: Option<String>,
}
