use sqlx::PgPool;
use crate::models::damage_notify_settings::{DamageNotifySettings, UpdateDamageNotifyRequest};

pub async fn get_or_create(pool: &PgPool, server_id: i32) -> Result<DamageNotifySettings, sqlx::Error> {
    let existing = sqlx::query_as::<_, DamageNotifySettings>(
        "SELECT * FROM damage_notify_settings WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await?;
    if let Some(s) = existing { return Ok(s); }
    sqlx::query_as::<_, DamageNotifySettings>(
        "INSERT INTO damage_notify_settings (server_id) VALUES ($1) RETURNING *"
    ).bind(server_id).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: &UpdateDamageNotifyRequest) -> Result<DamageNotifySettings, sqlx::Error> {
    let c = get_or_create(pool, server_id).await?;
    sqlx::query_as::<_, DamageNotifySettings>(
        "UPDATE damage_notify_settings SET enabled=$1, keyword=$2, min_damage=$3, notify_tk=$4, \
         notify_damage=$5, notify_high_damage=$6, high_damage_threshold=$7, updated_at=NOW() \
         WHERE server_id=$8 RETURNING *"
    )
    .bind(req.enabled.unwrap_or(c.enabled))
    .bind(req.keyword.as_deref().unwrap_or(&c.keyword))
    .bind(req.min_damage.unwrap_or(c.min_damage))
    .bind(req.notify_tk.unwrap_or(c.notify_tk))
    .bind(req.notify_damage.unwrap_or(c.notify_damage))
    .bind(req.notify_high_damage.unwrap_or(c.notify_high_damage))
    .bind(req.high_damage_threshold.unwrap_or(c.high_damage_threshold))
    .bind(server_id)
    .fetch_one(pool).await
}

