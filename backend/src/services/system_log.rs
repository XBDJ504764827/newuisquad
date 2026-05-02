use sqlx::PgPool;

pub async fn log(pool: &PgPool, log_type: &str, level: &str, module: &str, message: &str, detail: &str) {
    let _ = sqlx::query(
        "INSERT INTO system_logs (log_type, level, module, message, detail) VALUES ($1,$2,$3,$4,$5)"
    ).bind(log_type).bind(level).bind(module).bind(message).bind(detail).execute(pool).await;
}

pub async fn backend_info(pool: &PgPool, module: &str, message: &str) {
    log(pool, "backend", "INFO", module, message, "").await;
}
pub async fn backend_warn(pool: &PgPool, module: &str, message: &str) {
    log(pool, "backend", "WARN", module, message, "").await;
}
pub async fn agent_event(pool: &PgPool, module: &str, message: &str) {
    log(pool, "agent", "INFO", module, message, "").await;
}
pub async fn action_log(pool: &PgPool, module: &str, message: &str, detail: &str) {
    log(pool, "action", "INFO", module, message, detail).await;
}
