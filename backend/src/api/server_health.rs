use axum::{Json, extract::State, http::StatusCode};
use crate::api::AppState;
use crate::services::server_monitor;
use crate::services::server_monitor::HealthStatus;

/// GET /api/v1/servers/enhanced
/// 增强版服务器列表（含健康状态 + 24h 统计）
pub async fn list_enhanced(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let monitor = server_monitor::ServerMonitor::new(state.pool.clone(), state.rcon_pool.clone());

    let pt = state.game_services.player_tracker.as_deref();
    match server_monitor::get_enhanced_server_list(&state.pool, &monitor, pt).await {
        Ok(servers) => Ok(Json(serde_json::json!({ "data": servers }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": format!("查询失败: {}", e) }))),
    }
}

/// GET /api/v1/servers/{id}/health
/// 单个服务器健康状态
pub async fn server_health(
    State(state): State<AppState>,
    axum::extract::Path(server_id): axum::extract::Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref monitor) = state.game_services.server_monitor {
        let health = monitor.get_health(server_id).await;
        return Ok(Json(serde_json::json!(health)));
    }
    Ok(Json(serde_json::json!({ "error": "ServerMonitor 未启用" })))
}

/// GET /api/v1/servers/{id}/stats
/// 服务器 24h 统计
pub async fn server_stats(
    State(state): State<AppState>,
    axum::extract::Path(server_id): axum::extract::Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match server_monitor::get_24h_stats(&state.pool, server_id).await {
        Ok(stats) => Ok(Json(serde_json::json!(stats))),
        Err(e) => Ok(Json(serde_json::json!({ "error": format!("查询失败: {}", e) }))),
    }
}

/// GET /api/v1/servers-health
/// 所有服务器健康状态汇总
pub async fn all_health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref monitor) = state.game_services.server_monitor {
        let states = monitor.get_all_health().await;
        let online = states.values().filter(|h| h.status == server_monitor::HealthStatus::Online).count();
        let degraded = states.values().filter(|h| h.status == server_monitor::HealthStatus::Degraded).count();
        let offline = states.values().filter(|h| h.status == server_monitor::HealthStatus::Offline).count();

        return Ok(Json(serde_json::json!({
            "servers": states.values().collect::<Vec<_>>(),
            "summary": {
                "total": states.len(),
                "online": online,
                "degraded": degraded,
                "offline": offline,
            }
        })));
    }
    Ok(Json(serde_json::json!({ "error": "ServerMonitor 未启用" })))
}
