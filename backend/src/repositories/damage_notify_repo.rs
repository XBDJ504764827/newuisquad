use sqlx::PgPool;
use crate::models::damage_notify_settings::{DamageNotifySettings, UpdateDamageNotifyRequest};

pub async fn get_or_create(pool: &PgPool, server_id: i32) -> Result<DamageNotifySettings, sqlx::Error> {
    let existing = sqlx::query_as::<_, DamageNotifySettings>(
        "SELECT id, server_id, enabled, notify_kill, notify_damage, message_mode, hit_layout, kill_layout, updated_at \
         FROM damage_notify_settings WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await?;
    if let Some(s) = existing { return Ok(s); }
    sqlx::query_as::<_, DamageNotifySettings>(
        "INSERT INTO damage_notify_settings (server_id) VALUES ($1) \
         RETURNING id, server_id, enabled, notify_kill, notify_damage, message_mode, hit_layout, kill_layout, updated_at"
    ).bind(server_id).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: &UpdateDamageNotifyRequest) -> Result<DamageNotifySettings, sqlx::Error> {
    let c = get_or_create(pool, server_id).await?;
    sqlx::query_as::<_, DamageNotifySettings>(
        "UPDATE damage_notify_settings SET \
         enabled=$1, notify_kill=$2, notify_damage=$3, message_mode=$4, hit_layout=$5, kill_layout=$6, updated_at=NOW() \
         WHERE server_id=$7 \
         RETURNING id, server_id, enabled, notify_kill, notify_damage, message_mode, hit_layout, kill_layout, updated_at"
    )
    .bind(req.enabled.unwrap_or(c.enabled))
    .bind(req.notify_kill.unwrap_or(c.notify_kill))
    .bind(req.notify_damage.unwrap_or(c.notify_damage))
    .bind(req.message_mode.as_deref().unwrap_or(&c.message_mode))
    .bind(req.hit_layout.as_deref().unwrap_or(&c.hit_layout))
    .bind(req.kill_layout.as_deref().unwrap_or(&c.kill_layout))
    .bind(server_id)
    .fetch_one(pool).await
}
