use axum::{Json, extract::{State, Query}, http::StatusCode};
use serde::Deserialize;
use crate::api::AppState;

#[derive(Deserialize, Default)]
pub struct LogQuery {
    pub log_type: Option<String>,
    pub server_id: Option<i32>,
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
        if lt == "action" {
            audit_logs(&state, q.server_id, per_page, offset).await?
        } else if lt == "agent" {
            agent_logs(&state, q.server_id, per_page, offset).await?
        } else {
            system_logs(&state, lt, per_page, offset).await?
        }
    } else {
        all_logs(&state, q.server_id, page, per_page, offset).await?
    };

    Ok(Json(serde_json::json!({ "data": items, "total": total, "page": page, "per_page": per_page })))
}

async fn system_logs(
    state: &AppState, log_type: &str, per_page: i64, offset: i64,
) -> Result<(i64, Vec<serde_json::Value>), StatusCode> {
    let total = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM system_logs WHERE log_type=$1")
        .bind(log_type).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.0;
    let rows = sqlx::query_as::<_, (String, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT log_type, level, module, message, detail, logged_at FROM system_logs WHERE log_type=$1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
    ).bind(log_type).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data: Vec<serde_json::Value> = rows.into_iter().map(|(lt, level, module, msg, detail, ts)| {
        serde_json::json!({"log_type": lt, "level": level, "module": module, "message": msg, "detail": detail, "logged_at": ts, "source": "system"})
    }).collect();
    Ok((total, data))
}

async fn agent_logs(
    state: &AppState, _server_id: Option<i32>, per_page: i64, offset: i64,
) -> Result<(i64, Vec<serde_json::Value>), StatusCode> {
    let total = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM server_logs WHERE server_id > 0")
        .fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.0;
    let rows = sqlx::query_as::<_, (i32, String, Option<String>, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT server_id, log_level, category, message, logged_at FROM server_logs ORDER BY logged_at DESC LIMIT $1 OFFSET $2"
    ).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data: Vec<serde_json::Value> = rows.into_iter().map(|(sid, level, cat, msg, ts)| {
        serde_json::json!({"log_type": "agent", "server_id": sid, "level": level, "module": cat, "message": msg, "logged_at": ts, "source": "agent"})
    }).collect();
    Ok((total, data))
}

async fn audit_logs(
    state: &AppState, server_id: Option<i32>, per_page: i64, offset: i64,
) -> Result<(i64, Vec<serde_json::Value>), StatusCode> {
    let mut all: Vec<serde_json::Value> = Vec::new();

    // 1. system_logs (action type)
    let sys_rows = sqlx::query_as::<_, (String, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT log_type, level, module, message, detail, logged_at FROM system_logs WHERE log_type='action' ORDER BY logged_at DESC LIMIT $1"
    ).bind(per_page).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for (lt, level, module, msg, detail, ts) in sys_rows {
        all.push(serde_json::json!({"log_type": lt, "level": level, "module": module, "message": msg, "detail": detail, "logged_at": ts, "source": "system"}));
    }

    // 2. rcon_logs (参数化查询)
    let rcon_rows = if let Some(sid) = server_id {
        sqlx::query_as::<_, (i32, i32, String, String, String, Option<String>, chrono::DateTime<chrono::Utc>)>(
            "SELECT r.id, r.server_id, s.name, r.admin_user, r.command, r.response, r.created_at FROM rcon_logs r JOIN servers s ON r.server_id=s.id WHERE r.server_id=$1 ORDER BY r.created_at DESC LIMIT $2"
        ).bind(sid).bind(per_page).fetch_all(&state.pool).await
    } else {
        sqlx::query_as::<_, (i32, i32, String, String, String, Option<String>, chrono::DateTime<chrono::Utc>)>(
            "SELECT r.id, r.server_id, s.name, r.admin_user, r.command, r.response, r.created_at FROM rcon_logs r JOIN servers s ON r.server_id=s.id ORDER BY r.created_at DESC LIMIT $1"
        ).bind(per_page).fetch_all(&state.pool).await
    }.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for (_id, sid, sname, admin, cmd, resp, ts) in rcon_rows {
        let resp_preview = resp.as_deref().unwrap_or("");
        all.push(serde_json::json!({
            "log_type": "action", "level": "INFO", "module": "RCON",
            "message": format!("{} 在服务器 {} 执行: {}", admin, sname, cmd),
            "detail": if resp_preview.len() > 200 { &resp_preview[..200] } else { resp_preview },
            "logged_at": ts, "source": "rcon",
            "server_id": sid, "admin_user": admin,
        }));
    }

    // 3. admin_actions (参数化查询)
    let admin_rows = if let Some(sid) = server_id {
        sqlx::query_as::<_, (i32, i32, String, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT a.id, a.server_id, s.name, a.admin_name, a.action_type, a.target, a.message, a.logged_at FROM admin_actions a JOIN servers s ON a.server_id=s.id WHERE a.server_id=$1 ORDER BY a.logged_at DESC LIMIT $2"
        ).bind(sid).bind(per_page).fetch_all(&state.pool).await
    } else {
        sqlx::query_as::<_, (i32, i32, String, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT a.id, a.server_id, s.name, a.admin_name, a.action_type, a.target, a.message, a.logged_at FROM admin_actions a JOIN servers s ON a.server_id=s.id ORDER BY a.logged_at DESC LIMIT $1"
        ).bind(per_page).fetch_all(&state.pool).await
    }.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for (_id, _sid, _sname, admin, action, target, msg, ts) in admin_rows {
        let action_label = match action.as_str() {
            "warn" => "警告", "kick" => "踢出", "ban" => "封禁",
            "broadcast" => "广播", "change_layer" => "切换地图",
            _ => &action,
        };
        all.push(serde_json::json!({
            "log_type": "action", "level": "WARN",
            "module": "管理员操作",
            "message": format!("{} {} {}", admin, action_label, target),
            "detail": msg,
            "logged_at": ts, "source": "admin_action",
            "server_id": _sid, "admin_user": admin,
        }));
    }

    all.sort_by(|a, b| b["logged_at"].as_str().cmp(&a["logged_at"].as_str()));
    let total = all.len() as i64;
    let skip = (offset as usize).min(all.len());
    let take = (per_page as usize).min(all.len().saturating_sub(skip));
    let page_data = all[skip..skip + take].to_vec();

    Ok((total, page_data))
}

async fn all_logs(
    state: &AppState, _server_id: Option<i32>, _page: i64, per_page: i64, offset: i64,
) -> Result<(i64, Vec<serde_json::Value>), StatusCode> {
    // 串行查询三个来源，内存排序
    let mut all: Vec<serde_json::Value> = Vec::new();

    // system_logs
    let sys_rows = sqlx::query_as::<_, (String, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT log_type, level, module, message, detail, logged_at FROM system_logs ORDER BY logged_at DESC LIMIT 500"
    ).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for (lt, level, module, msg, detail, ts) in sys_rows {
        all.push(serde_json::json!({"log_type": lt, "level": level, "module": module, "message": msg, "detail": detail, "logged_at": ts, "source": "system"}));
    }

    // server_logs (agent)
    let agent_rows = sqlx::query_as::<_, (i32, String, Option<String>, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT server_id, log_level, category, message, logged_at FROM server_logs ORDER BY logged_at DESC LIMIT 500"
    ).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for (sid, level, cat, msg, ts) in agent_rows {
        all.push(serde_json::json!({"log_type": "agent", "server_id": sid, "level": level, "module": cat, "message": msg, "logged_at": ts, "source": "agent"}));
    }

    all.sort_by(|a, b| b["logged_at"].as_str().cmp(&a["logged_at"].as_str()));
    let total = all.len() as i64;
    let skip = (offset as usize).min(all.len());
    let take = (per_page as usize).min(all.len().saturating_sub(skip));
    let page_data = all[skip..skip + take].to_vec();

    Ok((total, page_data))
}
