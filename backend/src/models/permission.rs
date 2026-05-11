use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    #[serde(default)]
    pub group_name: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_permissions")]
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
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_permissions")]
    pub permissions: Option<String>,
    pub parent_group_id: Option<Option<i32>>,
    pub is_admin: Option<bool>,
}

fn deserialize_permissions<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(permission_value_to_string(value).unwrap_or_default())
}

fn deserialize_optional_permissions<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(permission_value_to_string(value))
}

fn permission_value_to_string(value: Option<Value>) -> Option<String> {
    match value {
        Some(Value::Array(items)) => Some(
            items
                .into_iter()
                .filter_map(|item| item.as_str().map(str::trim).map(str::to_string))
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
                .join(","),
        ),
        Some(Value::String(value)) => Some(value),
        Some(Value::Null) | None => None,
        Some(_) => Some(String::new()),
    }
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
