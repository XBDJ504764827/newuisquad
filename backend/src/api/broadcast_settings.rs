use axum::{Json, extract::{State, Path}, http::StatusCode};
use crate::api::AppState;
use crate::models::broadcast_settings::UpdateBroadcastRequest;
use crate::services::broadcast_service;

pub async fn get(State(state): State<AppState>, Path(server_id): Path<i32>) -> Result<Json<serde_json::Value>, StatusCode> {
    let s = broadcast_service::get(&state.pool, server_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!(s)))
}
pub async fn update(State(state): State<AppState>, Path(server_id): Path<i32>, Json(req): Json<UpdateBroadcastRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    let s = broadcast_service::update(&state.pool, server_id, req).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!(s)))
}
