use axum::{
    Json, extract::{State, Path, Query, ws::{WebSocket, WebSocketUpgrade, Message}},
    http::StatusCode, response::IntoResponse,
};
use serde::Deserialize;

use crate::api::AppState;
use crate::services::log_service;

#[derive(Deserialize)]
pub struct LogQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(query): Query<LogQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);
    let result = log_service::get_logs(&state.pool, server_id, page, per_page)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!(result)))
}

pub async fn stream(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state, server_id))
}

async fn handle_ws(mut socket: WebSocket, state: AppState, server_id: i32) {
    let Some(tx) = &state.log_broadcast else {
        let _ = socket.send(Message::Text("{\"error\":\"日志监听未启动\"}".into())).await;
        return;
    };

    let mut rx = tx.subscribe();
    loop {
        match rx.recv().await {
            Ok(entry) => {
                if entry.server_id != server_id {
                    continue;
                }
                let json = serde_json::to_string(&entry).unwrap_or_default();
                if socket.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(_) => break,
        }
    }
}
