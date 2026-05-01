use axum::{Json, extract::{State, Path}, http::StatusCode};
use crate::api::AppState;
use crate::models::seed_settings::UpdateSeedSettings;
use crate::repositories::seed_settings_repo;

pub async fn get(State(state): State<AppState>, Path(sid): Path<i32>) -> Result<Json<serde_json::Value>, StatusCode> {
    let s = seed_settings_repo::get_or_create(&state.pool, sid).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!(s)))
}
pub async fn update(State(state): State<AppState>, Path(sid): Path<i32>, Json(req): Json<UpdateSeedSettings>) -> Result<Json<serde_json::Value>, StatusCode> {
    let s = seed_settings_repo::update(&state.pool, sid, &req).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!(s)))
}
