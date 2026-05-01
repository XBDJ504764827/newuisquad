use sqlx::PgPool;
use crate::models::tk_settings::{TkSettings, UpdateTkSettingsRequest};
use crate::repositories::tk_settings_repo;

pub async fn get(pool: &PgPool, server_id: i32) -> Result<TkSettings, sqlx::Error> {
    tk_settings_repo::get_or_create(pool, server_id).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: UpdateTkSettingsRequest) -> Result<TkSettings, sqlx::Error> {
    tk_settings_repo::update(pool, server_id, &req).await
}
