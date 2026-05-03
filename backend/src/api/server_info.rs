use axum::{Json, extract::{State, Path}, http::StatusCode};
use crate::api::AppState;
use crate::models::server::UpdateServerRequest;
use crate::services::server_service;

pub async fn list(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let servers = server_service::list(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": servers })))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let server = server_service::get(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match server {
        Some(s) => Ok(Json(serde_json::json!(s))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateServerRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let server = server_service::update(&state.pool, id, req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match server {
        Some(s) => Ok(Json(serde_json::json!(s))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn summary(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let pool = &state.pool;

    let (player_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT steam64) FROM player_info WHERE server_id=$1 AND last_seen >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (error_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM server_logs WHERE server_id=$1 AND log_level='ERROR' AND logged_at >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (warn_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM server_logs WHERE server_id=$1 AND log_level='WARN' AND logged_at >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (match_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM match_info WHERE server_id=$1 AND logged_at >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (kill_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM kill_events WHERE server_id=$1 AND is_kill=true AND logged_at >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (tk_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM kill_events WHERE server_id=$1 AND is_teamkill=true AND logged_at >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let latest_match = sqlx::query_as::<_, (String, String, String, String, Option<i32>, chrono::DateTime<chrono::Utc>)>(
        "SELECT map_name, layer_name, team1_faction, team2_faction, winner_team, logged_at FROM match_info WHERE server_id=$1 ORDER BY logged_at DESC LIMIT 1"
    ).bind(server_id).fetch_optional(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let recent_errors: Vec<serde_json::Value> = sqlx::query_as::<_, (String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT log_level, message, logged_at FROM server_logs WHERE server_id=$1 AND (log_level='ERROR' OR log_level='WARN') ORDER BY logged_at DESC LIMIT 10"
    ).bind(server_id).fetch_all(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.into_iter().map(|(level, msg, ts)| {
        serde_json::json!({ "level": level, "message": msg, "logged_at": ts })
    }).collect();

    Ok(Json(serde_json::json!({
        "player_count_24h": player_count,
        "error_count_24h": error_count,
        "warn_count_24h": warn_count,
        "match_count_24h": match_count,
        "kill_count_24h": kill_count,
        "tk_count_24h": tk_count,
        "latest_match": latest_match.map(|(map, layer, t1, t2, winner, ts)| {
            serde_json::json!({ "map_name": map, "layer_name": layer, "team1_faction": t1, "team2_faction": t2, "winner_team": winner, "logged_at": ts })
        }),
        "recent_errors": recent_errors,
    })))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!(server_id = id, "收到删除服务器请求");
    // 记录服务器名称用于审计日志
    let server_name = sqlx::query_scalar::<_, String>("SELECT name FROM servers WHERE id=$1")
        .bind(id).fetch_optional(&state.pool).await.ok().flatten().unwrap_or_default();
    match server_service::delete(&state.pool, id).await {
        Ok(true) => {
            crate::services::system_log::action_log(&state.pool, "servers", &format!("删除服务器 {}", server_name), &format!("server_id={}", id)).await;
            tracing::info!(server_id = id, "服务器删除成功");
            Ok(Json(serde_json::json!({ "ok": true })))
        }
        Ok(false) => {
            tracing::warn!(server_id = id, "服务器不存在");
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!(server_id = id, error = %e, "删除服务器失败");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
