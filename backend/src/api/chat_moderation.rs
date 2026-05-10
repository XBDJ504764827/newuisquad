use axum::{Json, extract::{State, Path, Query}, http::StatusCode};
use serde::Deserialize;
use crate::api::AppState;
use crate::services::chat_automod;

#[derive(Deserialize)]
pub struct UpdateModerationRequest {
    pub enabled: Option<bool>,
    pub enable_racial_slurs: Option<bool>,
    pub enable_homophobic_slurs: Option<bool>,
    pub enable_ableist_language: Option<bool>,
    pub enable_chinese_slurs: Option<bool>,
    pub custom_blacklist: Option<Vec<String>>,
    pub whitelist: Option<Vec<String>>,
    pub escalation_actions: Option<Vec<chat_automod::EscalationAction>>,
    pub violation_expiry_days: Option<i32>,
    pub exempt_admins: Option<bool>,
    pub log_detections: Option<bool>,
}

#[derive(Deserialize, Default)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// GET /api/v1/servers/{id}/chat-moderation-settings
pub async fn get_settings(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row = sqlx::query_as::<_, (i32, bool, bool, bool, bool, bool, Vec<String>, Vec<String>, serde_json::Value, i32, bool, bool)>(
        "SELECT server_id, enabled, enable_racial_slurs, enable_homophobic_slurs, enable_ableist_language, \
         enable_chinese_slurs, custom_blacklist, whitelist, escalation_actions, \
         violation_expiry_days, exempt_admins, log_detections \
         FROM chat_moderation_settings WHERE server_id=$1"
    ).bind(server_id).fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let settings = match row {
        Some((sid, enabled, racial, homo, ableist, chinese, blacklist, whitelist, actions, expiry, exempt, log)) => {
            let escalation: Vec<chat_automod::EscalationAction> = serde_json::from_value(actions).unwrap_or_default();
            chat_automod::ChatModerationSettings {
                id: 0, server_id: sid, enabled,
                enable_racial_slurs: racial,
                enable_homophobic_slurs: homo,
                enable_ableist_language: ableist,
                enable_chinese_slurs: chinese,
                custom_blacklist: blacklist,
                whitelist,
                escalation_actions: escalation,
                violation_expiry_days: expiry,
                exempt_admins: exempt,
                log_detections: log,
            }
        }
        None => {
            // Auto-create default settings
            let _ = sqlx::query(
                "INSERT INTO chat_moderation_settings (server_id) VALUES ($1) ON CONFLICT DO NOTHING"
            ).bind(server_id).execute(&state.pool).await;
            chat_automod::ChatModerationSettings {
                id: 0, server_id, enabled: false,
                enable_racial_slurs: true,
                enable_homophobic_slurs: true,
                enable_ableist_language: true,
                enable_chinese_slurs: true,
                custom_blacklist: vec![],
                whitelist: vec![],
                escalation_actions: vec![],
                violation_expiry_days: 30,
                exempt_admins: true,
                log_detections: true,
            }
        }
    };

    Ok(Json(serde_json::json!(settings)))
}

/// PUT /api/v1/servers/{id}/chat-moderation-settings
pub async fn update_settings(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Json(req): Json<UpdateModerationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Ensure row exists
    let _ = sqlx::query(
        "INSERT INTO chat_moderation_settings (server_id) VALUES ($1) ON CONFLICT DO NOTHING"
    ).bind(server_id).execute(&state.pool).await;

    // Get current
    let row = sqlx::query_as::<_, (bool, bool, bool, bool, bool, Vec<String>, Vec<String>, serde_json::Value, i32, bool, bool)>(
        "SELECT enabled, enable_racial_slurs, enable_homophobic_slurs, enable_ableist_language, \
         enable_chinese_slurs, custom_blacklist, whitelist, escalation_actions, \
         violation_expiry_days, exempt_admins, log_detections \
         FROM chat_moderation_settings WHERE server_id=$1"
    ).bind(server_id).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let enabled = req.enabled.unwrap_or(row.0);
    let racial = req.enable_racial_slurs.unwrap_or(row.1);
    let homo = req.enable_homophobic_slurs.unwrap_or(row.2);
    let ableist = req.enable_ableist_language.unwrap_or(row.3);
    let chinese = req.enable_chinese_slurs.unwrap_or(row.4);
    let blacklist = req.custom_blacklist.unwrap_or(row.5);
    let whitelist = req.whitelist.unwrap_or(row.6);
    let actions = req.escalation_actions.map(|a| serde_json::to_value(a).unwrap_or(row.7.clone())).unwrap_or(row.7);
    let expiry = req.violation_expiry_days.unwrap_or(row.8);
    let exempt = req.exempt_admins.unwrap_or(row.9);
    let log = req.log_detections.unwrap_or(row.10);

    let _ = sqlx::query(
        "UPDATE chat_moderation_settings SET enabled=$1, enable_racial_slurs=$2, enable_homophobic_slurs=$3, \
         enable_ableist_language=$4, enable_chinese_slurs=$5, custom_blacklist=$6, whitelist=$7, \
         escalation_actions=$8, violation_expiry_days=$9, exempt_admins=$10, log_detections=$11, updated_at=NOW() \
         WHERE server_id=$12"
    ).bind(enabled).bind(racial).bind(homo).bind(ableist).bind(chinese)
     .bind(&blacklist).bind(&whitelist).bind(&actions).bind(expiry).bind(exempt).bind(log)
     .bind(server_id).execute(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Reload automod settings
    if let Some(ref automod) = state.chat_automod {
        automod.write().await.load_settings(&state.pool).await;
    }

    get_settings(State(state), Path(server_id)).await
}

/// GET /api/v1/servers/{id}/chat-violations
pub async fn list_violations(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).min(200);

    match chat_automod::list_violations(&state.pool, server_id, page, per_page).await {
        Ok((records, total)) => Ok(Json(serde_json::json!({
            "data": records, "total": total, "page": page, "per_page": per_page
        }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": format!("查询失败: {}", e) }))),
    }
}
