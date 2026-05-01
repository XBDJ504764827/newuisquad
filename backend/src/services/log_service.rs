use sqlx::PgPool;
use std::path::PathBuf;
use tokio::sync::broadcast;
use crate::models::server_log::{LogEntry, LogPage};
use crate::repositories::server_log_repo;

pub fn start_log_watcher(
    file_path: PathBuf,
    pool: PgPool,
    server_id: i32,
) -> broadcast::Sender<LogEntry> {
    crate::log_watcher::watcher::start_watching(file_path, pool, server_id)
}

pub async fn get_logs(
    pool: &PgPool,
    server_id: i32,
    page: i64,
    per_page: i64,
) -> Result<LogPage, sqlx::Error> {
    server_log_repo::list_logs(pool, server_id, page, per_page).await
}
