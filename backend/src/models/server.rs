use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Server {
    pub id: i32,
    pub server_id: String,
    pub name: String,
    pub ip: String,
    pub rcon_port: i32,
    #[serde(skip_serializing)]
    pub rcon_password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub ip: Option<String>,
    pub rcon_port: Option<i32>,
    pub rcon_password: Option<String>,
}
