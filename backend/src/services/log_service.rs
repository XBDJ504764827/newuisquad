use sqlx::PgPool;
use crate::models::server_log::LogPage;
use crate::repositories::server_log_repo;

pub async fn get_logs(
    pool: &PgPool,
    server_id: i32,
    page: i64,
    per_page: i64,
) -> Result<LogPage, sqlx::Error> {
    server_log_repo::list_logs(pool, server_id, page, per_page).await
}
