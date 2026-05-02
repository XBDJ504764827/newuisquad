use axum::{Json, extract::{State, Path, Query}, http::StatusCode};
use serde::Deserialize;
use crate::api::AppState;
use crate::models::abnormal_damage::{UpdateAbnormalDamageConfigRequest, CreateAbnormalDamageRule, AbnormalDamageLog};
use crate::services::abnormal_damage_service;

#[derive(Deserialize, Default)]
pub struct LogsQuery {
    pub player_name: Option<String>,
    pub steamid64: Option<String>,
    pub limit: Option<i64>,
}

pub async fn get_config(State(state): State<AppState>, Path(server_id): Path<i32>) -> Result<Json<serde_json::Value>, StatusCode> {
    let s = abnormal_damage_service::get_config(&state.pool, server_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!(s)))
}

pub async fn update_config(State(state): State<AppState>, Path(server_id): Path<i32>, Json(req): Json<UpdateAbnormalDamageConfigRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    let s = abnormal_damage_service::update_config(&state.pool, server_id, req).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!(s)))
}

pub async fn list_rules(State(state): State<AppState>, Path(server_id): Path<i32>) -> Result<Json<serde_json::Value>, StatusCode> {
    let items = sqlx::query_as::<_, crate::models::abnormal_damage::AbnormalDamageRule>(
        "SELECT * FROM abnormal_damage_rules WHERE server_id=$1 ORDER BY id"
    ).bind(server_id).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn create_rule(State(state): State<AppState>, Path(server_id): Path<i32>, Json(req): Json<CreateAbnormalDamageRule>) -> Result<Json<serde_json::Value>, StatusCode> {
    let item = sqlx::query_as::<_, crate::models::abnormal_damage::AbnormalDamageRule>(
        "INSERT INTO abnormal_damage_rules (server_id, max_damage) VALUES ($1, $2) RETURNING *"
    ).bind(server_id).bind(req.max_damage).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!(item)))
}

pub async fn delete_rule(State(state): State<AppState>, Path((server_id, id)): Path<(i32, i32)>) -> Result<Json<serde_json::Value>, StatusCode> {
    sqlx::query("DELETE FROM abnormal_damage_rules WHERE id=$1 AND server_id=$2").bind(id).bind(server_id)
        .execute(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn list_logs(State(state): State<AppState>, Path(server_id): Path<i32>, Query(q): Query<LogsQuery>) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = q.limit.unwrap_or(100).min(500);
    let items: Vec<AbnormalDamageLog> = if let Some(ref name) = q.player_name {
        sqlx::query_as::<_, AbnormalDamageLog>(
            "SELECT * FROM abnormal_damage_logs WHERE server_id=$1 AND player_name ILIKE $2 ORDER BY logged_at DESC LIMIT $3"
        ).bind(server_id).bind(format!("%{}%", name)).bind(limit).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else if let Some(ref sid) = q.steamid64 {
        sqlx::query_as::<_, AbnormalDamageLog>(
            "SELECT * FROM abnormal_damage_logs WHERE server_id=$1 AND player_steamid64=$2 ORDER BY logged_at DESC LIMIT $3"
        ).bind(server_id).bind(sid).bind(limit).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as::<_, AbnormalDamageLog>(
            "SELECT * FROM abnormal_damage_logs WHERE server_id=$1 ORDER BY logged_at DESC LIMIT $2"
        ).bind(server_id).bind(limit).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    Ok(Json(serde_json::json!({ "data": items, "total": items.len() })))
}
