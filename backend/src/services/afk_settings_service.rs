use sqlx::PgPool;
use crate::models::afk_settings::{AfkSettings, UpdateAfkSettingsRequest};
use crate::repositories::afk_settings_repo;

pub async fn get(pool: &PgPool, server_id: i32) -> Result<AfkSettings, sqlx::Error> {
    afk_settings_repo::get_or_create(pool, server_id).await
}
pub async fn update(pool: &PgPool, server_id: i32, req: UpdateAfkSettingsRequest) -> Result<AfkSettings, sqlx::Error> {
    afk_settings_repo::update(pool, server_id, &req).await
}
