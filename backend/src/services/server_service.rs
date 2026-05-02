use sqlx::PgPool;
use crate::models::server::{Server, UpdateServerRequest};
use crate::repositories::server_repo;

pub async fn list(pool: &PgPool) -> Result<Vec<Server>, sqlx::Error> {
    server_repo::list_servers(pool).await
}

pub async fn get(pool: &PgPool, id: i32) -> Result<Option<Server>, sqlx::Error> {
    server_repo::get_server(pool, id).await
}

pub async fn update(pool: &PgPool, id: i32, req: UpdateServerRequest) -> Result<Option<Server>, sqlx::Error> {
    server_repo::update_server(pool, id, &req).await
}

pub async fn delete(pool: &PgPool, id: i32) -> Result<bool, sqlx::Error> {
    server_repo::delete_server(pool, id).await
}
