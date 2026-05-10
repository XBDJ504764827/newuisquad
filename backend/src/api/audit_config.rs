use axum::{Json, extract::{State, Path, Query}, http::StatusCode};
use serde::Deserialize;
use sqlx::PgPool;
use crate::api::AppState;

#[derive(Deserialize, Default)]
pub struct AuditQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub server_id: Option<i32>,
    pub admin_user: Option<String>,
    pub action_type: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

// ════════════════════════════════════════════
//  审计统计 API
// ════════════════════════════════════════════

/// GET /api/v1/audit-stats
pub async fn audit_stats(
    State(state): State<AppState>,
    Query(q): Query<AuditQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let pool = &state.pool;
    let server_filter = q.server_id.map(|s| format!("AND server_id = {}", s)).unwrap_or_default();
    let now = chrono::Utc::now();
    let days_ago = q.date_from.as_deref().unwrap_or("7");
    let days: i32 = days_ago.parse().unwrap_or(7);

    // RCON commands count
    let (rcon_count,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM rcon_logs WHERE created_at >= NOW() - INTERVAL '{} days' {}",
        days, server_filter
    )).fetch_one(pool).await.unwrap_or((0,));

    // Admin actions count
    let (action_count,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM admin_actions WHERE logged_at >= NOW() - INTERVAL '{} days' {}",
        days, server_filter
    )).fetch_one(pool).await.unwrap_or((0,));

    // Chat violations count
    let (violation_count,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM chat_violations WHERE logged_at >= NOW() - INTERVAL '{} days' {}",
        days, server_filter
    )).fetch_one(pool).await.unwrap_or((0,));

    // System logs count
    let (sys_err_count,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM system_logs WHERE logged_at >= NOW() - INTERVAL '{} days' AND level='ERROR' {}",
        days, if server_filter.is_empty() { "".to_string() } else { server_filter.replace("server_id", "1") }
    )).fetch_one(pool).await.unwrap_or((0,));

    // Unique admins
    let (unique_admins,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(DISTINCT admin_user) FROM rcon_logs WHERE created_at >= NOW() - INTERVAL '{} days' {}",
        days, server_filter
    )).fetch_one(pool).await.unwrap_or((0,));

    // Action breakdown
    let action_breakdown = sqlx::query_as::<_, (String, i64)>(&format!(
        "SELECT action_type, COUNT(*) as cnt FROM admin_actions \
         WHERE logged_at >= NOW() - INTERVAL '{} days' {} \
         GROUP BY action_type ORDER BY cnt DESC",
        days, server_filter
    )).fetch_all(pool).await.unwrap_or_default();

    // Daily breakdown
    let daily = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(&format!(
        "SELECT DATE(logged_at) as d, COUNT(*) as cnt FROM admin_actions \
         WHERE logged_at >= NOW() - INTERVAL '{} days' {} \
         GROUP BY d ORDER BY d",
        days, server_filter
    )).fetch_all(pool).await.unwrap_or_default();

    let daily_data: Vec<serde_json::Value> = daily.into_iter().map(|(d, c)| {
        serde_json::json!({ "date": d.format("%Y-%m-%d").to_string(), "count": c })
    }).collect();

    let breakdown: Vec<serde_json::Value> = action_breakdown.into_iter().map(|(action, cnt)| {
        serde_json::json!({ "action": action, "count": cnt })
    }).collect();

    Ok(Json(serde_json::json!({
        "period_days": days,
        "rcon_commands": rcon_count,
        "admin_actions": action_count,
        "chat_violations": violation_count,
        "system_errors": sys_err_count,
        "unique_admins": unique_admins,
        "action_breakdown": breakdown,
        "daily_trend": daily_data,
    })))
}

/// GET /api/v1/audit-detail
/// Unified audit detail with enhanced filtering
pub async fn audit_detail(
    State(state): State<AppState>,
    Query(q): Query<AuditQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let pool = &state.pool;
    let mut conditions = Vec::new();
    let mut params: Vec<String> = Vec::new();
    let mut param_idx = 0u32;

    if let Some(ref sid) = q.server_id.map(|s| s.to_string()) {
        param_idx += 1;
        conditions.push(format!("server_id = ${}", param_idx));
        params.push(sid.clone());
    }
    if let Some(ref admin) = q.admin_user {
        param_idx += 1;
        conditions.push(format!("admin_user ILIKE ${}", param_idx));
        params.push(format!("%{}%", admin));
    }
    if let Some(ref action) = q.action_type {
        param_idx += 1;
        conditions.push(format!("action_type = ${}", param_idx));
        params.push(action.clone());
    }
    if let Some(ref from) = q.date_from {
        param_idx += 1;
        conditions.push(format!("logged_at >= ${}::timestamptz", param_idx));
        params.push(from.clone());
    }
    if let Some(ref to) = q.date_to {
        param_idx += 1;
        conditions.push(format!("logged_at <= ${}::timestamptz", param_idx));
        params.push(to.clone());
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Build dynamic query
    let query_base = format!(
        "SELECT id, server_id, admin_name as admin_user, action_type, target, message, logged_at, 'admin_action' as source \
         FROM admin_actions {}",
        where_clause
    );

    // For simplicity, use a fixed query with optional filters
    let mut all: Vec<serde_json::Value> = Vec::new();

    // Query admin_actions with filters
    let action_query = if let Some(ref sid) = q.server_id {
        if let Some(ref admin) = q.admin_user {
            sqlx::query_as::<_, (i32, i32, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
                "SELECT id, server_id, admin_name, action_type, target, message, logged_at FROM admin_actions \
                 WHERE server_id=$1 AND admin_name ILIKE $2 ORDER BY logged_at DESC LIMIT 500"
            ).bind(sid).bind(format!("%{}%", admin)).fetch_all(pool).await
        } else {
            sqlx::query_as::<_, (i32, i32, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
                "SELECT id, server_id, admin_name, action_type, target, message, logged_at FROM admin_actions \
                 WHERE server_id=$1 ORDER BY logged_at DESC LIMIT 500"
            ).bind(sid).fetch_all(pool).await
        }
    } else {
        sqlx::query_as::<_, (i32, i32, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, server_id, admin_name, action_type, target, message, logged_at FROM admin_actions \
             ORDER BY logged_at DESC LIMIT 500"
        ).fetch_all(pool).await
    }.unwrap_or_default();

    for (id, sid, admin, action, target, msg, ts) in action_query {
        all.push(serde_json::json!({
            "id": id, "server_id": sid, "admin_user": admin,
            "action_type": action, "target": target, "message": msg,
            "logged_at": ts, "source": "admin_action",
        }));
    }

    // Add RCON logs
    let rcon_query = if let Some(ref sid) = q.server_id {
        sqlx::query_as::<_, (i32, i32, String, String, String, Option<String>, chrono::DateTime<chrono::Utc>)>(
            "SELECT r.id, r.server_id, r.admin_user, r.command, '' as target, r.response, r.created_at \
             FROM rcon_logs r WHERE r.server_id=$1 ORDER BY r.created_at DESC LIMIT 500"
        ).bind(sid).fetch_all(pool).await
    } else {
        sqlx::query_as::<_, (i32, i32, String, String, String, Option<String>, chrono::DateTime<chrono::Utc>)>(
            "SELECT r.id, r.server_id, r.admin_user, r.command, '' as target, r.response, r.created_at \
             FROM rcon_logs r ORDER BY r.created_at DESC LIMIT 500"
        ).fetch_all(pool).await
    }.unwrap_or_default();

    for (id, sid, admin, cmd, _target, resp, ts) in rcon_query {
        all.push(serde_json::json!({
            "id": id, "server_id": sid, "admin_user": admin,
            "action_type": "RCON", "target": cmd,
            "message": resp.as_deref().unwrap_or("").chars().take(200).collect::<String>(),
            "logged_at": ts, "source": "rcon_log",
        }));
    }

    // Sort and paginate
    all.sort_by(|a, b| {
        b["logged_at"].as_str().cmp(&a["logged_at"].as_str())
    });

    let total = all.len() as i64;
    let skip = (offset as usize).min(all.len());
    let take = (per_page as usize).min(all.len().saturating_sub(skip));
    let page_data = all[skip..skip + take].to_vec();

    Ok(Json(serde_json::json!({
        "data": page_data, "total": total, "page": page, "per_page": per_page
    })))
}

// ════════════════════════════════════════════
//  配置文件历史 API
// ════════════════════════════════════════════

#[derive(Deserialize)]
pub struct ConfigHistoryQuery {
    pub file_path: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// GET /api/v1/servers/{id}/config-history
pub async fn config_history(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<ConfigHistoryQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (total, rows) = if let Some(ref path) = q.file_path {
        let (t,) = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM config_file_history WHERE server_id=$1 AND file_path=$2"
        ).bind(server_id).bind(path).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let rows = sqlx::query_as::<_, (String, i32, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT file_path, version, admin_user, created_at FROM config_file_history \
             WHERE server_id=$1 AND file_path=$2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
        ).bind(server_id).bind(path).bind(per_page).bind(offset)
         .fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        (t, rows)
    } else {
        let (t,) = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM config_file_history WHERE server_id=$1"
        ).bind(server_id).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let rows = sqlx::query_as::<_, (String, i32, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT file_path, version, admin_user, created_at FROM config_file_history \
             WHERE server_id=$1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        ).bind(server_id).bind(per_page).bind(offset)
         .fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        (t, rows)
    };

    let data: Vec<serde_json::Value> = rows.into_iter().map(|(path, ver, admin, ts)| {
        serde_json::json!({ "file_path": path, "version": ver, "admin_user": admin, "created_at": ts })
    }).collect();

    Ok(Json(serde_json::json!({ "data": data, "total": total, "page": page, "per_page": per_page })))
}

/// GET /api/v1/servers/{id}/config-history/{version}
pub async fn config_history_detail(
    State(state): State<AppState>,
    Path((server_id, version)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row = sqlx::query_as::<_, (String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT file_path, content, admin_user, created_at FROM config_file_history \
         WHERE server_id=$1 AND version=$2 ORDER BY created_at DESC LIMIT 1"
    ).bind(server_id).bind(version).fetch_optional(&state.pool).await
     .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some((path, content, admin, ts)) => Ok(Json(serde_json::json!({
            "server_id": server_id, "version": version,
            "file_path": path, "content": content, "admin_user": admin, "created_at": ts,
        }))),
        None => Ok(Json(serde_json::json!({ "error": "版本不存在" }))),
    }
}

/// Save config history (called from files.rs write handler)
pub async fn save_config_history(
    pool: &PgPool,
    server_id: i32,
    file_path: &str,
    content: &str,
    admin_user: &str,
) {
    // Get next version
    let next_ver = sqlx::query_scalar::<_, i32>(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM config_file_history WHERE server_id=$1 AND file_path=$2"
    ).bind(server_id).bind(file_path).fetch_one(pool).await.unwrap_or(1);

    let _ = sqlx::query(
        "INSERT INTO config_file_history (server_id, file_path, content, admin_user, version) VALUES ($1,$2,$3,$4,$5)"
    ).bind(server_id).bind(file_path).bind(content).bind(admin_user).bind(next_ver)
     .execute(pool).await;
}
