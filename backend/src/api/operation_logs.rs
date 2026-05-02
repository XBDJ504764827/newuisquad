use axum::{Json, extract::{State, Query}, http::StatusCode};
use serde::Deserialize;
use crate::api::AppState;

#[derive(Deserialize, Default)]
pub struct LogQuery {
    pub log_type: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<LogQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let (total, items) = if let Some(ref lt) = q.log_type {
        // agent 日志 = server_logs 中的特定分类
        if lt == "agent" {
            let total = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM server_logs WHERE server_id > 0")
                .fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.0;
            let items = sqlx::query_as::<_, (i32, String, Option<String>, String, chrono::DateTime<chrono::Utc>)>(
                "SELECT server_id, log_level, category, message, logged_at FROM server_logs ORDER BY logged_at DESC LIMIT $1 OFFSET $2"
            ).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let data: Vec<serde_json::Value> = items.into_iter().map(|(sid, level, cat, msg, ts)| {
                serde_json::json!({"server_id": sid, "level": level, "category": cat, "message": msg, "logged_at": ts, "log_type": "agent"})
            }).collect();
            (total, data)
        } else {
            // backend/action 日志
            let total = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM system_logs WHERE log_type=$1")
                .bind(lt).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.0;
            let items = sqlx::query_as::<_, (String, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
                "SELECT log_type, level, module, message, detail, logged_at FROM system_logs WHERE log_type=$1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
            ).bind(lt).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let data: Vec<serde_json::Value> = items.into_iter().map(|(lt2, level, module, msg, detail, ts)| {
                serde_json::json!({"log_type": lt2, "level": level, "module": module, "message": msg, "detail": detail, "logged_at": ts})
            }).collect();
            (total, data)
        }
    } else {
        // 全部：system_logs + server_logs（最近1000条）
        let total_sys = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM system_logs")
            .fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.0;
        let total_agent = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM server_logs WHERE server_id > 0")
            .fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.0;
        let total = total_sys + total_agent;

        let sys_items = sqlx::query_as::<_, (String, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT log_type, level, module, message, detail, logged_at FROM system_logs ORDER BY logged_at DESC LIMIT 500"
        ).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let agent_items = sqlx::query_as::<_, (i32, String, Option<String>, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT server_id, log_level, category, message, logged_at FROM server_logs ORDER BY logged_at DESC LIMIT 500"
        ).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut data: Vec<serde_json::Value> = Vec::new();
        for (lt, level, module, msg, detail, ts) in sys_items {
            data.push(serde_json::json!({"log_type": lt, "level": level, "module": module, "message": msg, "detail": detail, "logged_at": ts}));
        }
        for (sid, level, cat, msg, ts) in agent_items {
            data.push(serde_json::json!({"log_type": "agent", "server_id": sid, "level": level, "category": cat, "message": msg, "logged_at": ts}));
        }
        data.sort_by(|a, b| b["logged_at"].as_str().cmp(&a["logged_at"].as_str()));
        data.truncate(((page * per_page) as usize).min(data.len()));
        let skip = (offset as usize).min(data.len());
        data = data[skip..].to_vec();
        (total, data)
    };

    Ok(Json(serde_json::json!({ "data": items, "total": total, "page": page, "per_page": per_page })))
}
