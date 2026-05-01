use sqlx::PgPool;
use crate::models::admin_user::AdminUser;

pub async fn list(pool: &PgPool) -> Result<Vec<AdminUser>, sqlx::Error> {
    sqlx::query_as::<_, AdminUser>("SELECT * FROM admin_users ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<AdminUser>, sqlx::Error> {
    sqlx::query_as::<_, AdminUser>("SELECT * FROM admin_users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(
    pool: &PgPool, username: &str, password_hash: &str, role: &str,
    permissions: &serde_json::Value, steam_id64: Option<&str>, notes: Option<&str>,
) -> Result<AdminUser, sqlx::Error> {
    sqlx::query_as::<_, AdminUser>(
        "INSERT INTO admin_users (username, password_hash, role, permissions, steam_id64, notes) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
    )
    .bind(username).bind(password_hash).bind(role).bind(permissions)
    .bind(steam_id64).bind(notes)
    .fetch_one(pool)
    .await
}

pub async fn update(
    pool: &PgPool, id: i32, username: Option<&str>, password_hash: Option<&str>,
    role: Option<&str>, permissions: Option<&serde_json::Value>,
    steam_id64: Option<&str>, notes: Option<&str>,
) -> Result<Option<AdminUser>, sqlx::Error> {
    let current = get_by_id(pool, id).await?;
    let Some(c) = current else { return Ok(None) };

    sqlx::query_as::<_, AdminUser>(
        "UPDATE admin_users SET username=$1, password_hash=$2, role=$3, permissions=$4, steam_id64=$5, notes=$6, updated_at=NOW() WHERE id=$7 RETURNING *"
    )
    .bind(username.unwrap_or(&c.username))
    .bind(password_hash.unwrap_or(&c.password_hash))
    .bind(role.unwrap_or(&c.role))
    .bind(permissions.unwrap_or(&c.permissions))
    .bind(steam_id64.or(c.steam_id64.as_deref()))
    .bind(notes.or(c.notes.as_deref()))
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: i32) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM admin_users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
