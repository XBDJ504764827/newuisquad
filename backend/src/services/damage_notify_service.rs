use sqlx::PgPool;
use crate::models::damage_notify_settings::{DamageNotifySettings, UpdateDamageNotifyRequest};
use crate::repositories::damage_notify_repo;

pub async fn get(pool: &PgPool, server_id: i32) -> Result<DamageNotifySettings, sqlx::Error> {
    damage_notify_repo::get_or_create(pool, server_id).await
}
pub async fn update(pool: &PgPool, server_id: i32, req: UpdateDamageNotifyRequest) -> Result<DamageNotifySettings, sqlx::Error> {
    damage_notify_repo::update(pool, server_id, &req).await
}
