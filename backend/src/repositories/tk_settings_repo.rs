use sqlx::PgPool;
use crate::models::tk_settings::{TkSettings, UpdateTkSettingsRequest};

pub async fn get_or_create(pool: &PgPool, server_id: i32) -> Result<TkSettings, sqlx::Error> {
    let existing = sqlx::query_as::<_, TkSettings>("SELECT * FROM tk_settings WHERE server_id = $1")
        .bind(server_id)
        .fetch_optional(pool)
        .await?;
    if let Some(s) = existing { return Ok(s); }
    sqlx::query_as::<_, TkSettings>(
        "INSERT INTO tk_settings (server_id) VALUES ($1) RETURNING *"
    ).bind(server_id).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: &UpdateTkSettingsRequest) -> Result<TkSettings, sqlx::Error> {
    let current = get_or_create(pool, server_id).await?;
    sqlx::query_as::<_, TkSettings>(
        "UPDATE tk_settings SET enabled=$1, max_team_kills=$2, apology_time_minutes=$3, notification_message=$4, updated_at=NOW() WHERE server_id=$5 RETURNING *"
    )
    .bind(req.enabled.unwrap_or(current.enabled))
    .bind(req.max_team_kills.unwrap_or(current.max_team_kills))
    .bind(req.apology_time_minutes.unwrap_or(current.apology_time_minutes))
    .bind(req.notification_message.as_deref().unwrap_or(current.notification_message.as_deref().unwrap_or("")))
    .bind(server_id)
    .fetch_one(pool)
    .await
}
