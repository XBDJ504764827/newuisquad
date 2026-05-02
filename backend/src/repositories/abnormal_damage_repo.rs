use sqlx::PgPool;
use crate::models::abnormal_damage::{AbnormalDamageConfig, UpdateAbnormalDamageConfigRequest};

pub async fn get_or_create(pool: &PgPool, server_id: i32) -> Result<AbnormalDamageConfig, sqlx::Error> {
    let existing = sqlx::query_as::<_, AbnormalDamageConfig>(
        "SELECT * FROM abnormal_damage_config WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await?;
    if let Some(s) = existing { return Ok(s); }
    sqlx::query_as::<_, AbnormalDamageConfig>(
        "INSERT INTO abnormal_damage_config (server_id) VALUES ($1) RETURNING *"
    ).bind(server_id).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: &UpdateAbnormalDamageConfigRequest) -> Result<AbnormalDamageConfig, sqlx::Error> {
    let c = get_or_create(pool, server_id).await?;
    sqlx::query_as::<_, AbnormalDamageConfig>(
        "UPDATE abnormal_damage_config SET enabled=$1, updated_at=NOW() WHERE server_id=$2 RETURNING *"
    )
    .bind(req.enabled.unwrap_or(c.enabled))
    .bind(server_id)
    .fetch_one(pool).await
}
