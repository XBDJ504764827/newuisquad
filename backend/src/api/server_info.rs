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

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!(server_id = id, "收到删除服务器请求");
    match server_service::delete(&state.pool, id).await {
        Ok(true) => {
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
