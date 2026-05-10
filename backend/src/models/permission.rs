use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PermissionGroup {
    pub id: i32,
    pub server_id: i32,
    pub group_name: String,
    pub permissions: String,
    pub parent_group_id: Option<i32>,
    pub is_admin: bool,
    pub is_template: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Simplified row for internal resolution (avoids timestamp parsing issues)
#[derive(Debug, Clone)]
pub struct PermissionGroupRow {
    pub id: i32,
    pub server_id: i32,
    pub group_name: String,
    pub permissions: String,
    pub parent_group_id: Option<i32>,
    pub is_admin: bool,
    pub is_template: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreatePermissionGroupRequest {
    pub group_name: String,
    pub permissions: String,
    #[serde(default)]
    pub parent_group_id: Option<i32>,
    #[serde(default = "default_true")]
    pub is_admin: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Deserialize)]
pub struct UpdatePermissionGroupRequest {
    pub group_name: Option<String>,
    pub permissions: Option<String>,
    pub parent_group_id: Option<Option<i32>>,
    pub is_admin: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PermissionAdmin {
    pub id: i32,
    pub server_id: i32,
    pub steam_id: String,
    pub group_name: String,
    pub player_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePermissionAdminRequest {
    pub steam_id: String,
    pub group_name: String,
    #[serde(default)]
    pub player_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePermissionAdminRequest {
    pub group_name: Option<String>,
    pub player_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BanRecord {
    pub id: i32,
    pub server_id: i32,
    pub steam_id: String,
    pub player_name: String,
    pub duration: i32,
    pub reason: String,
    pub admin_user: String,
    pub created_at: DateTime<Utc>,
}

/// Resolved permissions result with inheritance info
#[derive(Debug, Clone, Serialize)]
pub struct ResolvedPermissions {
    pub group_name: String,
    pub permissions: Vec<String>,
    pub inherited_from: Vec<String>,
}
