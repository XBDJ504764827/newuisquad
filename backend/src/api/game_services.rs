use axum::{Json, extract::{State, Path}, http::StatusCode};
use crate::api::AppState;
use crate::services::afk_service::AfkPlayerInfo;
use crate::services::afk_service::AfkStatus;
use crate::services::command_catalog;
use crate::services::motd_generator::{MotdGenerator, MotdConfig, ServerRule};
use crate::services::identity_resolver::IdentityResolver;

/// GET /api/v1/servers/{id}/seed-status
pub async fn seed_status(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref svc) = state.seeding_service {
        if let Some(mode) = svc.get_mode(server_id).await {
            return Ok(Json(serde_json::json!({
                "server_id": server_id,
                "mode": format!("{:?}", mode),
            })));
        }
    }
    Ok(Json(serde_json::json!({ "server_id": server_id, "mode": "unknown" })))
}

/// GET /api/v1/servers/{id}/team-balance-status
pub async fn balance_status(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref svc) = state.team_balance {
        if let Some(s) = svc.get_state(server_id).await {
            return Ok(Json(serde_json::json!({
                "server_id": server_id,
                "enabled": s.config.enabled,
                "win_streak_team": s.win_streak_team,
                "win_streak_count": s.win_streak_count,
                "last_scramble": s.last_scramble,
                "scramble_in_progress": s.scramble_in_progress,
            })));
        }
    }
    Ok(Json(serde_json::json!({ "server_id": server_id, "enabled": false })))
}

/// POST /api/v1/servers/{id}/scramble
/// 手动触发阵营洗牌
pub async fn manual_scramble(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref svc) = state.team_balance {
        let result = svc.manual_scramble(server_id).await;
        return Ok(Json(serde_json::json!(result)));
    }
    Ok(Json(serde_json::json!({ "error": "队伍平衡服务未启用" })))
}

/// GET /api/v1/servers/{id}/afk-status
pub async fn afk_status(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref svc) = state.afk_service {
        let states = svc.get_state(server_id).await;
        let now = chrono::Utc::now();
        let players: Vec<AfkPlayerInfo> = states.into_iter().map(|s| AfkPlayerInfo {
            player_name: s.player_name,
            player_id: s.player_id,
            afk_minutes: (now - s.unassigned_since).num_minutes() as i32,
            warning_count: s.warning_count,
            kicked: s.kicked,
        }).collect();
        return Ok(Json(serde_json::json!({
            "server_id": server_id,
            "players": players,
            "total_afk": players.len(),
        })));
    }
    Ok(Json(serde_json::json!({ "server_id": server_id, "players": [], "total_afk": 0 })))
}

/// GET /api/v1/command-catalog
pub async fn command_catalog_list() -> Result<Json<serde_json::Value>, StatusCode> {
    let commands = command_catalog::command_catalog();
    let categories: std::collections::HashMap<String, Vec<&command_catalog::CommandInfo>> = std::collections::HashMap::new();
    // Group by category
    let mut cat_map = categories;
    for cmd in &commands {
        cat_map.entry(cmd.category.clone()).or_default().push(cmd);
    }
    Ok(Json(serde_json::json!({
        "commands": commands,
        "total": commands.len(),
    })))
}

/// POST /api/v1/motd/preview
/// Preview MOTD from rules
#[derive(serde::Deserialize)]
pub struct MotdPreviewRequest {
    pub rules: Vec<ServerRule>,
    #[serde(default)]
    pub prefix_text: String,
    #[serde(default)]
    pub suffix_text: String,
    #[serde(default = "default_true")]
    pub include_descriptions: bool,
}
fn default_true() -> bool { true }

pub async fn motd_preview(
    Json(req): Json<MotdPreviewRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let config = MotdConfig {
        server_id: 0,
        prefix_text: req.prefix_text,
        suffix_text: req.suffix_text,
        auto_generate_from_rules: true,
        include_rule_descriptions: req.include_descriptions,
    };
    let motd = MotdGenerator::generate(&config, &req.rules);
    let rule_count = MotdGenerator::count_rules(&req.rules);
    Ok(Json(serde_json::json!({
        "motd": motd,
        "rule_count": rule_count,
    })))
}

/// GET /api/v1/servers/{id}/motd-preview
/// Preview MOTD using server-stored rules (from DB)
pub async fn server_motd_preview(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get server info for prefix
    let server_name = sqlx::query_scalar::<_, String>("SELECT name FROM servers WHERE id=$1")
        .bind(server_id).fetch_optional(&state.pool).await
        .ok().flatten().unwrap_or_default();

    let config = MotdConfig {
        server_id,
        prefix_text: format!("=== {} 服务器规则 ===\n", server_name),
        suffix_text: "\n=== 祝您游戏愉快 ===".to_string(),
        auto_generate_from_rules: true,
        include_rule_descriptions: true,
    };

    // For now, use sample empty rules (rules CRUD would be a separate feature)
    let motd = MotdGenerator::generate(&config, &[]);
    Ok(Json(serde_json::json!({
        "motd": motd,
        "rule_count": 0,
    })))
}

/// POST /api/v1/identity/compute
/// 执行身份识别计算
pub async fn compute_identities(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match IdentityResolver::compute(&state.pool).await {
        Ok(count) => Ok(Json(serde_json::json!({ "success": true, "identities_found": count }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}

/// GET /api/v1/identity/lookup?q={steam_id|eos_id|name}
#[derive(serde::Deserialize)]
pub struct IdentityQuery { pub q: String }

pub async fn identity_lookup(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<IdentityQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match IdentityResolver::lookup(&state.pool, &q.q).await {
        Ok(Some(identity)) => Ok(Json(serde_json::json!(identity))),
        Ok(None) => Ok(Json(serde_json::json!({ "error": "未找到身份" }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}

/// GET /api/v1/identities?page=1&per_page=50
#[derive(serde::Deserialize, Default)]
pub struct IdentityListQuery { pub page: Option<i64>, pub per_page: Option<i64> }

pub async fn identity_list(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<IdentityListQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).min(200);
    match IdentityResolver::list_all(&state.pool, page, per_page).await {
        Ok((identities, total)) => Ok(Json(serde_json::json!({
            "data": identities, "total": total, "page": page, "per_page": per_page
        }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}
