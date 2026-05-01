use sqlx::PgPool;
use crate::models::rcon_log::RconLog;

pub async fn insert_rcon_log(
    pool: &PgPool,
    server_id: i32,
    admin_user: &str,
    command: &str,
    response: &str,
) -> Result<RconLog, sqlx::Error> {
    sqlx::query_as::<_, RconLog>(
        "INSERT INTO rcon_logs (server_id, admin_user, command, response) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(server_id).bind(admin_user).bind(command).bind(response)
    .fetch_one(pool)
    .await
}

pub async fn list_rcon_logs(pool: &PgPool, server_id: i32, limit: i64) -> Result<Vec<RconLog>, sqlx::Error> {
    sqlx::query_as::<_, RconLog>(
        "SELECT * FROM rcon_logs WHERE server_id = $1 ORDER BY created_at DESC LIMIT $2"
    )
    .bind(server_id).bind(limit)
    .fetch_all(pool)
    .await
}
