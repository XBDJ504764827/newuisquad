use axum::{Json, extract::{State, Path}, http::StatusCode};
use crate::api::AppState;
use crate::models::announcement::CreateAnnouncement;

pub async fn list(State(state): State<AppState>, Path(server_id): Path<i32>) -> Result<Json<serde_json::Value>, StatusCode> {
    let items = sqlx::query_as::<_, crate::models::announcement::Announcement>(
        "SELECT * FROM announcements WHERE server_id=$1 ORDER BY id", ).bind(server_id)
        .fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn create(State(state): State<AppState>, Path(server_id): Path<i32>, Json(req): Json<CreateAnnouncement>) -> Result<Json<serde_json::Value>, StatusCode> {
    let item = sqlx::query_as::<_, crate::models::announcement::Announcement>(
        "INSERT INTO announcements (server_id,content,interval_minutes) VALUES ($1,$2,$3) RETURNING *"
    ).bind(server_id).bind(&req.content).bind(req.interval_minutes).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!(item)))
}

pub async fn delete(State(state): State<AppState>, Path((server_id, id)): Path<(i32, i32)>) -> Result<Json<serde_json::Value>, StatusCode> {
    sqlx::query("DELETE FROM announcements WHERE id=$1 AND server_id=$2").bind(id).bind(server_id)
        .execute(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "success": true })))
}
