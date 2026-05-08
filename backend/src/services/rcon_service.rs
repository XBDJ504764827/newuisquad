use sqlx::PgPool;
use crate::api::AppState;
use crate::models::rcon_log::{ExecuteRconRequest, RconLog};
use crate::repositories::{rcon_log_repo, server_repo};
use crate::services::system_log;

pub async fn execute(
    state: &AppState,
    server_id: i32,
    req: &ExecuteRconRequest,
) -> Result<RconLog, String> {
    let server = server_repo::get_server(&state.pool, server_id)
        .await
        .map_err(|e| format!("查询服务器失败: {}", e))?
        .ok_or_else(|| "服务器不存在".to_string())?;

    let response = state
        .rcon_pool
        .execute(
            &server.ip,
            server.rcon_port as u16,
            &server.rcon_password,
            &req.command,
        )
        .await?;

    system_log::backend_info(
        &state.pool,
        "rcon",
        &format!(
            "{} 在服务器 {} 执行: {}",
            req.admin_user, server.name, req.command
        ),
    )
    .await;

    rcon_log_repo::insert_rcon_log(&state.pool, server_id, &req.admin_user, &req.command, &response)
        .await
        .map_err(|e| format!("记录 RCON 日志失败: {}", e))
}

pub async fn list_logs(pool: &PgPool, server_id: i32) -> Result<Vec<RconLog>, sqlx::Error> {
    rcon_log_repo::list_rcon_logs(pool, server_id, 100).await
}
