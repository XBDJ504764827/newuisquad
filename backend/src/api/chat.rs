use axum::{Json, extract::{State, Path, Query}, http::StatusCode};
use serde::Deserialize;
use crate::api::AppState;

#[derive(Deserialize, Default)]
pub struct ChatQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub end: Option<String>,
}

fn parse_time(s: &Option<String>) -> Option<chrono::DateTime<chrono::Utc>> {
    s.as_deref()
        .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

pub async fn list(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<ChatQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(100).min(500);
    let offset = (page - 1) * per_page;
    let start = parse_time(&q.start);
    let end = parse_time(&q.end);

    let (total,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM chat_messages WHERE server_id=$1 AND ($2::timestamptz IS NULL OR logged_at >= $2) AND ($3::timestamptz IS NULL OR logged_at <= $3)"
    ).bind(server_id).bind(start).bind(end).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items = sqlx::query_as::<_, (i32, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT server_id, player_name, steam64, message, channel, logged_at FROM chat_messages WHERE server_id=$1 AND ($2::timestamptz IS NULL OR logged_at >= $2) AND ($3::timestamptz IS NULL OR logged_at <= $3) ORDER BY logged_at DESC LIMIT $4 OFFSET $5"
    ).bind(server_id).bind(start).bind(end).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data: Vec<serde_json::Value> = items.into_iter().map(|(sid, name, s64, msg, ch, ts)| {
        serde_json::json!({"server_id": sid, "player_name": name, "steam64": s64, "message": msg, "channel": ch, "logged_at": ts})
    }).collect();

    Ok(Json(serde_json::json!({ "data": data, "total": total, "page": page })))
}
