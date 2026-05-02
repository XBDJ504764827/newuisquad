use sqlx::PgPool;
use crate::models::abnormal_damage::{AbnormalDamageConfig, UpdateAbnormalDamageConfigRequest};
use crate::repositories::abnormal_damage_repo;

pub async fn get_config(pool: &PgPool, server_id: i32) -> Result<AbnormalDamageConfig, sqlx::Error> {
    abnormal_damage_repo::get_or_create(pool, server_id).await
}
pub async fn update_config(pool: &PgPool, server_id: i32, req: UpdateAbnormalDamageConfigRequest) -> Result<AbnormalDamageConfig, sqlx::Error> {
    abnormal_damage_repo::update(pool, server_id, &req).await
}
