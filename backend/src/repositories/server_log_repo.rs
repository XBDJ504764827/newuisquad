use sqlx::PgPool;
use crate::models::server_log::{LogPage, ServerLog, LogEntry};

pub async fn insert_log_entry(pool: &PgPool, server_id: i32, entry: &LogEntry) -> Result<ServerLog, sqlx::Error> {
    sqlx::query_as::<_, ServerLog>(
        "INSERT INTO server_logs (server_id, log_level, category, message, raw_line, logged_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
    )
    .bind(server_id).bind(&entry.log_level).bind(&entry.category).bind(&entry.message).bind(&entry.raw_line).bind(entry.logged_at)
    .fetch_one(pool)
    .await
}

pub async fn list_logs(
    pool: &PgPool,
    server_id: i32,
    page: i64,
    per_page: i64,
) -> Result<LogPage, sqlx::Error> {
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM server_logs WHERE server_id = $1"
    )
    .bind(server_id)
    .fetch_one(pool)
    .await?;

    let offset = (page - 1) * per_page;
    let data = sqlx::query_as::<_, ServerLog>(
        "SELECT * FROM server_logs WHERE server_id = $1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(server_id).bind(per_page).bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(LogPage { data, page, per_page, total: total.0 })
}
