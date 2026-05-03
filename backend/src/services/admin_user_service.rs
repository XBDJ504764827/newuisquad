use sqlx::PgPool;
use crate::models::admin_user::{AdminUser, CreateAdminRequest, UpdateAdminRequest};
use crate::repositories::admin_user_repo;

pub async fn list(pool: &PgPool) -> Result<Vec<AdminUser>, sqlx::Error> {
    admin_user_repo::list(pool).await
}

pub async fn create(pool: &PgPool, req: CreateAdminRequest) -> Result<AdminUser, String> {
    let hash = bcrypt::hash(&req.password, 10).map_err(|e| format!("密码加密失败: {}", e))?;
    let permissions = req.permissions.unwrap_or(serde_json::json!({}));
    admin_user_repo::create(
        pool, &req.username, &hash, &req.role, &permissions,
        req.steam_id64.as_deref(), req.notes.as_deref(),
    )
    .await
    .map_err(|e| format!("创建失败: {}", e))
}

pub async fn update(pool: &PgPool, id: i32, req: UpdateAdminRequest) -> Result<Option<AdminUser>, String> {
    let hash = req.password.map(|p| bcrypt::hash(&p, 10)).transpose().map_err(|e| format!("密码加密失败: {}", e))?;
    admin_user_repo::update(
        pool, id,
        req.username.as_deref(), hash.as_deref(),
        req.role.as_deref(), req.permissions.as_ref(),
        req.steam_id64.as_deref(), req.notes.as_deref(),
    )
    .await
    .map_err(|e| format!("更新失败: {}", e))
}

pub async fn delete(pool: &PgPool, id: i32) -> Result<bool, sqlx::Error> {
    admin_user_repo::delete(pool, id).await
}
