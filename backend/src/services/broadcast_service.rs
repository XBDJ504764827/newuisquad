use sqlx::PgPool;
use crate::models::broadcast_settings::{BroadcastSettings, UpdateBroadcastRequest};
use crate::repositories::broadcast_repo;

pub async fn get(pool: &PgPool, s: i32) -> Result<BroadcastSettings, sqlx::Error> { broadcast_repo::get_or_create(pool, s).await }
pub async fn update(pool: &PgPool, s: i32, r: UpdateBroadcastRequest) -> Result<BroadcastSettings, sqlx::Error> { broadcast_repo::update(pool, s, &r).await }
