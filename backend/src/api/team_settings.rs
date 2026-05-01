use axum::{Json, extract::{State, Path}, http::StatusCode};
use crate::api::AppState;
use crate::models::team_settings::UpdateTeamSettings;
use crate::repositories::team_settings_repo;

async fn get_or_create_impl(pool: &sqlx::PgPool, sid: i32) -> Result<crate::models::team_settings::TeamSettings, sqlx::Error> {
    team_settings_repo::get_or_create(pool, sid).await
}

pub async fn get(State(state): State<AppState>, Path(server_id): Path<i32>) -> Result<Json<serde_json::Value>, StatusCode> {
    let s = get_or_create_impl(&state.pool, server_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!(s)))
}
pub async fn update(State(state): State<AppState>, Path(server_id): Path<i32>, Json(req): Json<UpdateTeamSettings>) -> Result<Json<serde_json::Value>, StatusCode> {
    let s = team_settings_repo::update(&state.pool, server_id, &req).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!(s)))
}
