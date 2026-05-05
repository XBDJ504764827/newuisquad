use axum::{Json, extract::{State, Path}, http::StatusCode};
use crate::api::AppState;
use crate::models::team_switch::UpdateTeamSwitchConfigRequest;
use crate::services::team_switch_service;

pub async fn get_config(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let s = team_switch_service::get_config(&state.pool, server_id)
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // 回填缓存
    state.team_switch_cache.write().await.insert(server_id, s.enabled);
    Ok(Json(serde_json::json!(s)))
}

pub async fn update_config(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Json(req): Json<UpdateTeamSwitchConfigRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let s = team_switch_service::update_config(&state.pool, server_id, req)
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // 更新缓存
    state.team_switch_cache.write().await.insert(server_id, s.enabled);
    Ok(Json(serde_json::json!(s)))
}
