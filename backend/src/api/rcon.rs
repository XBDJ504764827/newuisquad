use axum::{Json, extract::{State, Path}, http::StatusCode};
use crate::api::AppState;
use crate::models::rcon_log::ExecuteRconRequest;
use crate::services::rcon_service;

pub async fn execute(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Json(req): Json<ExecuteRconRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match rcon_service::execute(&state.pool, server_id, &req).await {
        Ok(log) => Ok(Json(serde_json::json!(log))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}

pub async fn list_logs(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let logs = rcon_service::list_logs(&state.pool, server_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": logs })))
}
