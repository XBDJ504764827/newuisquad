use axum::{Json, extract::{State, Path}, http::StatusCode};
use crate::api::AppState;
use crate::models::admin_user::{CreateAdminRequest, UpdateAdminRequest};
use crate::services::admin_user_service;

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let users = admin_user_service::list(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": users })))
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateAdminRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match admin_user_service::create(&state.pool, req).await {
        Ok(user) => Ok(Json(serde_json::json!(user))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateAdminRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match admin_user_service::update(&state.pool, id, req).await {
        Ok(Some(user)) => Ok(Json(serde_json::json!(user))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match admin_user_service::delete(&state.pool, id).await {
        Ok(true) => Ok(Json(serde_json::json!({ "success": true }))),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
