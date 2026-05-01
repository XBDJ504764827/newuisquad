use sqlx::PgPool;
use crate::models::afk_settings::{AfkSettings, UpdateAfkSettingsRequest};

pub async fn get_or_create(pool: &PgPool, server_id: i32) -> Result<AfkSettings, sqlx::Error> {
    let existing = sqlx::query_as::<_, AfkSettings>("SELECT * FROM afk_settings WHERE server_id = $1")
        .bind(server_id).fetch_optional(pool).await?;
    if let Some(s) = existing { return Ok(s); }
    sqlx::query_as::<_, AfkSettings>("INSERT INTO afk_settings (server_id) VALUES ($1) RETURNING *")
        .bind(server_id).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: &UpdateAfkSettingsRequest) -> Result<AfkSettings, sqlx::Error> {
    let c = get_or_create(pool, server_id).await?;
    sqlx::query_as::<_, AfkSettings>(
        "UPDATE afk_settings SET enabled=$1, min_players_to_check=$2, max_afk_minutes=$3, updated_at=NOW() WHERE server_id=$4 RETURNING *"
    )
    .bind(req.enabled.unwrap_or(c.enabled)).bind(req.min_players_to_check.unwrap_or(c.min_players_to_check))
    .bind(req.max_afk_minutes.unwrap_or(c.max_afk_minutes)).bind(server_id)
    .fetch_one(pool).await
}
